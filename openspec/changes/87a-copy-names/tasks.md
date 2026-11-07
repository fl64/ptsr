# Tasks: Pad copy names

Change 87 is complete and merged into `master` at `93362d9`.
The owner approved all three naming choices, activation, the implementation file plan and local signed-off commits.
Publication, merge and archive need separate permission.
Each completed task requires a successful `task check` and a separate signed-off local commit.

## 1. Specification preparation

- [x] 1.1 Record the initial behavior, defaults, open questions, deltas and owner checks in four deferred artifacts. Check the English text, requirements and project checks. Commit only these artifacts.
- [x] 1.2 Record the three owner decisions after completion of 87. Check the four documents and run `task check`. Commit the revision separately.
- [x] 1.3 Update the branch from `master` without rewriting its history. Run `task check` before the merge commit.

## 2. Activate and implement numbered copy names

Activation and code form one logical task because active deltas require real scenario tests in `task check`.

- [x] 2.1 Activate the four artifacts and add change metadata. Update project context. Implement full pickup names, canonical markers and smallest-free numbers across all banks. Use the same calculation for questions and placement. Test every delta scenario through keyboard and raw MIDI with production dispatch. Cover Unicode, slices, legacy and manual names, replacement, changed proposals, cancellation, refusal and undo. Test project and scratch save/load without format or sample changes. Retain settings, shared sample data and copy-sound assertions. Run `task check`, the build, strict OpenSpec checks and screen checks. Commit this naming behavior separately.

## 3. Owner checks and documentation

- [ ] 3.1 Update `docs/USER-STORIES.md` and `docs/LIVE-CHECKS.md` with the naming rules and separate keyboard, controller and listening procedures. Record actual automated results and their limits. Check the English text and links. Run `task check`. Commit the documentation separately.
- [ ] 3.2 Present the checks and local copy render to the owner. Wait for acceptance and separate publication, archive and merge permissions.

## Historical preparation verification

The original `task check` and `task openspec:check` passed on application base `a4789f4`.
That run passed 690 App, 80 input, 102 drawing and 18 benchmark-report tests.
Main-spec coverage did not cover the deferred naming scenarios.
Direct strict checks of the deferred proposal, delta and task structure had no findings.
The English lint had no findings. This does not establish full ASD dictionary compliance.

## Current verification

The owner-decision revision passed `task check` before commit `bcc270d`.
Local log: `target/87a-revision-check.log`. The command exited 0.
The base update passed `task check` before merge commit `03cc232`.
Local log: `target/87a-base-check.log`. The command exited 0.
These runs checked the old naming behavior, not the new implementation.
Earlier callback maximum-budget failures remain unresolved.
Automated checks open no live sound or MIDI devices. Hardware and listening results remain unverified.

## Implementation verification

`task check` exited 0 with 1117 workspace tests, including 780 App, 81 input and 129 drawing tests.
The App count includes 15 new naming tests. Keyboard, raw notes and learned CC buttons use the production profile and dispatcher.
Controller runs use MIDI for banks, copy, confirmation, clearing and undo. Save/load tests cover both projects and scratch work.
The active-delta audit passed all 17 scenario names, including the two existing settings and sample-sharing tests.
`task build` exited 0 and produced `target/release/ptsr`.
`task openspec:check` reported `Totals: 16 passed, 0 failed (16 items)`.
`task screens` reported `test result: ok. 4 passed; 0 failed` and generated 306 ANSI files at 120x40 and 80x26.
Local logs: `target/87a-final-check.log`, `target/87a-final-build.log`, `target/87a-final-openspec.log` and `target/87a-final-screens.log`.

The existing copy-sound regression passed. `renders/83a-pad-copy.wav` has the same SHA-256 before and after implementation:
`2a8e1b1647d7ac6e493c2d00dd319093823d09f1b3785b49924e475c69cc3bfd`.
No new render is necessary because sound processing stays unchanged.
Steady callback maxima were 36.50% wall and 27.11% thread CPU, with no blocks above 50% in this run.
These measurements do not resolve earlier maximum-budget failures.

The first test-helper revision failed to compile because it omitted a MIDI port argument.
The corrected helper then exposed its incorrect wraparound assumption. Bank selection stops at the ends.
The final helper uses the real previous/next commands, and the final full run passes without weaker assertions.
English review used STE-flavored descriptions and Strict procedures. Active artifacts have no lint findings.
Context additions have no hard findings. Four passive-voice suggestions describe approval states and verification limits.
This review does not establish full ASD dictionary compliance.

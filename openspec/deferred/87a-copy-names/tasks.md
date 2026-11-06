# Tasks: Pad copy names

This change stays deferred until change 87 is complete.
The owner authorized only specification preparation now.
Each completed task requires a successful `task check` and a separate signed-off local commit.
Publication, merge and archive require separate permission.

## 1. Specification preparation

- [x] 1.1 Record approved behavior, defaults, open questions, deltas and owner checks in four artifacts. Review the English text. Check the deferred proposal and delta directly. Run `task openspec:check` and `task check`. Commit only these four artifacts.

## 2. Activation after change 87

- [ ] 2.1 Resolve the design Open questions with the owner. Review this draft against the completed change 87 and agree implementation files before activation.

## 3. Name selection and placement

- [ ] 3.1 Implement one copy marker, base-name preservation and available-number selection across all banks. Test canonical markers, Unicode, slices, legacy names and gaps.
- [ ] 3.2 Connect the same name selection to questions and placement. Test changed proposals, pickup snapshots, replacement, cancellation, refusal and undo.
- [ ] 3.3 Run every shared copy scenario through keyboard and raw MIDI using actual profiles and production dispatch. Retain existing copy-settings and sound assertions.

## 4. Persistence and acceptance

- [ ] 4.1 Test stored names and later number selection through project and scratch save/load paths. Keep sample files and saved formats unchanged.
- [ ] 4.2 Map every delta scenario to a named test. Update existing user and manual-check documents. Run project checks and retain the copy-sound render regression.
- [ ] 4.3 Present separate keyboard and controller checks to the owner. Archive only after acceptance and main-spec scenario coverage pass.

## Verification status

Preparation verification: `task check` and `task openspec:check` pass on the unchanged application base `a4789f4`.
The suites pass: 690 App, 80 input, 102 drawing and 18 benchmark-report tests.
Main-spec scenario coverage passes. It does not cover the deferred naming scenarios.
Direct strict OpenSpec checks of this proposal, delta and task structure report no errors, warnings or information items.
The English review uses STE-flavored descriptions and Strict procedures. Its lint reports zero findings, without a full dictionary-compliance claim.
The preparation commit contains only these four artifacts. Change 87 stays separate.
The callback benchmark command passes. Earlier maximum-budget failures remain unresolved because sound processing stays unchanged.
Raw benchmark records remain local. Automated tests opened no live audio or MIDI devices.
No naming implementation, new scenario test, manual acceptance or listening result exists for this change.
The sampler still uses the current ` 2` naming rule.

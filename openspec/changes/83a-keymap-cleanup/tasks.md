## 0. Specifications only

- [x] 0.1 Record the owner's option A, same-branch exception and specifications-only
  approval on 2026-11-03. Keep 84 deferred and existing keys unchanged.
- [x] 0.2 Complete all eight deltas, retain current scenarios, record defaults and
  tests, update the two planning documents, validate strictly, run `task check` and
  commit the specification step. Do not start code without its implementation plan.

## 1. Key resolution and migration feedback

- [ ] 1.1 Read the implementation files and call sites in full before the code plan.
  Add explicit Shift command support with tests for both layouts, extended events,
  legacy uppercase input, Shift+Tab, pad velocity and unchanged Ctrl/Alt behavior.
  Use existing actions in parser fixtures; add no unimplemented command. Run
  `task check` and commit the input step.
- [ ] 1.2 Add the App moved-key table and effective-binding guards. Test all old keys,
  valid rebound J/L, owner actions, `none`, text, questions and learn. Add individual
  startup shadow warnings tied to actual changed defaults, with matching-owner-action
  and multiple-conflict tests. Keep owner files unchanged. Run `task check` and commit.

## 2. Existing key moves

- [ ] 2.1 Move microphone to Alt+R, normalize to Ctrl+G and hit chopping to H. On Pads,
  use J/L and arrows for values and U/O for banks with edit open or closed. Keep output,
  Space and all controller defaults. Update affected help and recording hints in this
  same step. Update recorded input events, not assertions that protect behavior.
  Test each new key, its unbound old-key notice, legacy microphone toggle, repeat
  filtering, recording exclusion, microphone linking and value/bank/undo behavior.
  Add the shipped-map consistency test and reasoned exceptions. Run `task check`,
  confirm X-Session regression tests remain unchanged, and commit the key moves.

## 3. Named Chop commands

- [ ] 3.1 Add `chop.hits_back` and `chop.assign_order` to Action parsing, the catalogue,
  App and the two learn views together with their real handlers. Bind Shift+H and Tab.
  Test threshold direction, clamp, initial and knob values, cuts, links, undo, status,
  assign guards and old `edit.param_next` / `chop.auto_mode_next` compatibility. Test
  named-key picks in F10 and raw MIDI buttons through dispatch. Update help in the
  same step; run `task check` and commit. Do not change hit detection or audio DSP.

## 4. Pad copy

- [ ] 4.1 Add the copy snapshot, distinct hand state, App command and `pad.copy` learn
  entry with Ctrl+C. Use existing snapshot/restore helpers and shared samples. Route
  keyboard and MIDI target hits to App without direct duplicate sound; keep original
  releases and project epochs. Test all copy scenarios, target questions and recheck,
  marks, source/target load guards, cancellation, old swap behavior, names, source links,
  persistence and one-step undo. Add the visible copy instructions in the same step.
  Test Ctrl+C as input, not quit, in both terminal event forms. Run `task check` and commit.
- [ ] 4.2 Make `renders/83a-pad-copy.wav` from the actual App copy and undo commands.
  Compare source/copy sound with full settings and matched velocity. Check the restored
  target after undo, finite samples, no clipping, expected silence, timing within one
  sample and click-free replacement of a sounding target. Run `task check` and commit.

## 5. Documents, verification and acceptance

- [ ] 5.1 Add named tests for every new delta scenario and check coverage before archive.
  Keep old scenario names that mention Ctrl+Y and L as explicit owner-remap tests;
  default H and U/O need their own tests. Do not skip failures to make the check pass.
  Run strict OpenSpec validation, `task check` and `task build`.
- [ ] 5.2 Update README, user stories, live checks, vision, backlog, project context and
  OpenSpec context with implemented keys, the branch exception and the render to hear.
  Replace stale unconfirmed-A text only in the approved documentation scope for that
  step. Preserve Downloads and owner configuration. Run `task check` and commit.
- [ ] 5.3 Ask the owner for separate keyboard and controller passes from the proposal,
  legacy-terminal checks where available, and listening to `renders/83a-pad-copy.wav`.
  Record actual acceptance; simulated MIDI does not prove the SMC-PAD hardware map.
- [ ] 5.4 Only after tests and owner acceptance, archive through `openspec archive`.
  Check the rebuilt main specs and scenario coverage, run strict validation and
  `task check`, then commit. Do not archive manually or start 84 early.

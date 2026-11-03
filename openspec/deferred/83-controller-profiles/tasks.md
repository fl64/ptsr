## Status: do not execute (2026-11-03)

These are the former tasks. Change 91 in `docs/plans/SMC-PAD-INTEGRATION.md` replaces
this plan. Carry useful research and test cases there, then remove this folder in 91.
No unchecked task below means that the old absolute-CC plan should be implemented.

## 1. SMC-PAD profile and emulation

- [ ] 1.1 Replace the invalid template with the documented normal Note/absolute-CC map.
  Add physical orientation, button and both knob-set tests. Run task check and commit.
- [ ] 1.2 Add raw MIDI dispatch and App tests for velocity, releases, screen and bank
  changes, takeover, undo and harmless unsupported messages. Run task check and commit.

## 2. X-Session physical controls

- X-Session physical defaults and single-controller tests shipped in archived change 82,
  not this deferred change. Use that map without restoring old button roles.
- [ ] 2.2 Test both controllers together, including separate takeover, gate release,
  save and restart. Run task check and commit.

## 3. Acceptance

- [ ] 3.1 Update user stories, binding tables and live checks with source limitations.
  Validate specs, run task check and commit.
- [ ] 3.2 Ask the owner to verify the SMC-PAD preset map and both physical controllers.

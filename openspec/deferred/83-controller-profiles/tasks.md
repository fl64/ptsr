## 1. SMC-PAD profile and emulation

- [ ] 1.1 Replace the invalid template with the documented normal Note/absolute-CC map.
  Add physical orientation, button and both knob-set tests. Run task check and commit.
- [ ] 1.2 Add raw MIDI dispatch and App tests for velocity, releases, screen and bank
  changes, takeover, undo and harmless unsupported messages. Run task check and commit.

## 2. X-Session physical controls

- [ ] 2.1 Document physical labels, add the stated fader and FX defaults, and preserve
  existing bindings and chop actions. Add first-connect and reset tests. Run task check and commit.
- [ ] 2.2 Test both controllers together, including separate takeover, gate release,
  save and restart. Run task check and commit.

## 3. Acceptance

- [ ] 3.1 Update user stories, binding tables and live checks with source limitations.
  Validate specs, run task check and commit.
- [ ] 3.2 Ask the owner to verify the SMC-PAD preset map and both physical controllers.

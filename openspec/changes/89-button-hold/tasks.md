# Tasks

## 1. Specify button gestures

- [x] Record timed gestures, profile syntax, learned bindings, and the setting in OpenSpec deltas.
- [x] Record the unsupported lost-release case and the unmeasured SMC-PAD profile limit.
- [x] Run `task openspec:check` and STE lint on changed English text. Commit the text step.

## 2. Track gestures and parse bindings

- [ ] Add a pure tracker with supplied times and no sleeps.
- [ ] Parse profile maps and keep plain strings immediate. Refuse unknown actions with a clear file and control message.
- [ ] Store tap and hold slots in learned maps. Read old strings as immediate actions without rewriting the user config.
- [ ] End held actions on release or disconnect. Keep direct pad releases and input order.
- [ ] Add tests named for each input-device scenario, using timed events and raw MIDI through the real profile and dispatch.
- [ ] Run `cargo fmt --all --check`, Clippy for changed crates, and `task test:changed`. Commit the code step.

## 3. Learn and set the hold time

- [ ] Add tap and hold selection to button learning. Keep pending, cancel, clear, and pad or bank runs correct.
- [ ] Add a hold-time row to F12 MIDI with persistence and invalid-value handling.
- [ ] Test each new MIDI-learn and settings scenario through the application path.
- [ ] Compare the F12 MIDI screen catalogue at 80x26 and 120x40 with the base.
- [ ] Run the required code checks. Commit this step.

## 4. Review and publish

- [ ] Update the affected plans and project status. Mark unmeasured SMC-PAD facts as unverified.
- [ ] Walk each proposal item against tests and the visible MIDI screen.
- [ ] Run `task test:fast`, `task specs:check`, and `task names` before push.
- [ ] Give the owner separate keyboard and controller checks and the listening step.
- [ ] After owner acceptance, archive the change, run `task specs:sync`, and complete the PR checks.

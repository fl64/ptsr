# Tasks

## 1. Specify confirmation

- [x] Record the `ui.confirm` contract in the keyboard, MIDI learn, and screen help deltas.
- [x] Record the owner's controller priority in the design.
- [x] Run `task openspec:check` and STE lint on changed English text. Commit the text step.

## 2. Route one confirm action

- [ ] Add the action, its GLOBAL learn row, short label, and shipped Enter binding.
- [ ] Route keyboard and raw MIDI confirm presses through the existing question and picker guards.
- [ ] Keep the old answer actions, owner bindings, text input, learning, and held pad releases.
- [ ] Add tests named for each new scenario. Use the production profile and dispatch for raw MIDI.
- [ ] Run `cargo fmt --all --check`, Clippy for changed crates, and `task test:changed`. Commit this code step.

## 3. Show the working answer

- [ ] Show `ui.confirm` only when an effective keyboard or connected controller route reaches the current question.
- [ ] Check the question catalogue at 80x26 and 120x40 against the base.
- [ ] Run the required code checks and commit this step.

## 4. Review and publish

- [ ] Update `docs/PROJECT.md` and the ordered roadmap before publication. Keep the pending `denormals` plan change in its own branch.
- [ ] Walk each item in the proposal's What Changes list against the program and catalogue.
- [ ] Run `task test:fast`, `task specs:check`, `task names`, and the PR text lint before push.
- [ ] Give the owner separate keyboard and controller checks. Record unverified physical checks.
- [ ] After owner acceptance, archive the change, run `task specs:sync`, and complete the PR checks.

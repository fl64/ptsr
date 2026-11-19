## 1. Specification

- [x] Record the exit failure, the owner decision and the cases to test.

## 2. Exit and save

- [ ] Make the exit decision after the save result is available.
- [ ] Keep direct keyboard Exit and the SMC-PAD Exit question.
- [ ] Use the existing A1 completion boundary before an exit save.
- [ ] Join a pending manual save for the same session.
- [ ] Retain the completed take, its Undo step and pending edits after a save refusal.
- [ ] Use the existing safe file replacement and manual retry paths.

## 3. Recovery question

- [ ] Show the session name, the reason, `Retry Save` and `Back`.
- [ ] Route both actions through the existing question controls.
- [ ] Keep the save reason available after its notice expires.
- [ ] Show `Session saved` after a successful retry. Keep the session open.
- [ ] Return to the same screen with the edits after Back.

## 4. Tests and verification

- [ ] Cover each scenario through keyboard events and raw SMC-PAD MIDI where input applies.
- [ ] Use disposable files and an induced write refusal. Compare the saved bytes.
- [ ] Cover a pending A1 save and an active Gate note, including loss of audio feedback.
- [ ] Compare the recovery question at 80x26 and 120x40 with the base catalogue.
- [ ] Run format, Clippy and `task test:changed` before the code commit.
- [ ] Verify the keyboard and SMC-PAD routes in an isolated session.

## 1. Specification

- [x] Record the exit failure, the owner decision and the cases to test.
- [x] Record owner approval for implementation on 2026-11-19.

## 2. Exit and save

- [x] Make the exit decision after the save result is available.
- [x] Keep direct keyboard Exit and the SMC-PAD Exit question.
- [x] Use the existing A1 completion boundary before an exit save.
- [x] Join a pending manual save for the same session.
- [x] Retain the completed take, its Undo step and pending edits after a save refusal.
- [x] Discard later capture events. Allow a new take when the audio thread ends the old take.
- [x] Use the existing safe file replacement and manual retry paths.

## 3. Recovery question

- [x] Show the session name, the reason, `Retry Save` and `Back`.
- [x] Route both actions through the existing question controls.
- [x] Keep the save reason available after its notice expires.
- [x] Show `Session saved` after a successful retry. Keep the session open.
- [x] Return to the same screen with the edits after Back.

## 4. Tests and verification

- [x] Cover each scenario through keyboard events and raw SMC-PAD MIDI where input applies.
- [x] Use disposable files and an induced write refusal. Compare the saved bytes.
- [x] Cover a pending A1 save and an active Gate note, including loss of audio feedback.
- [x] Compare the recovery question at 80x26 and 120x40 with the base catalogue.
- [x] Run format, Clippy and `task test:changed` before the code commit.
- [x] Verify the keyboard and SMC-PAD routes in an isolated session.
- [x] Run the engine tests, offline renders and callback benchmark.

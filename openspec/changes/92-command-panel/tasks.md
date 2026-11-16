# Tasks

## 1. Evidence and specification

- [x] 1.1 Inspect action, MIDI, binding, transport, drawing and test call sites. Record the reset contract and owner question in the design.
- [x] 1.2 Specify panel cells, button paths, question priority, transport pause, keyboard paths and screen states. Record edge cases before code.
- [x] 1.3 Align `docs/plans/SMC-PAD-INTEGRATION.md`, `docs/plans/CROSS-CUTTING.md` and `docs/VISION.md` with the chosen split between 92 and 93a. Update the planned manual checks.
- [x] 1.4 Run `task openspec:check` and STE lint for changed English text. Commit the specification step.

## 2. Panel configuration and input

- [x] 2.1 Add and validate `config/panel.toml`. Keep bad cells inactive and report the file and cell. Test all shipped action names and fixed cells.
- [x] 2.2 Route the Master second-bank controls through production MIDI and App dispatch. Preserve first-hit order, releases, saved owner bindings, explicit `none` and both first-bank note-off forms. Add raw MIDI tests named after the panel scenarios.
- [x] 2.3 Keep panel ownership per connection. Test a second controller, disconnect, reconnect, invalid shared state and a held Gate. Run required changed-crate checks and commit this working step.

## 3. Panel behavior and safety

- [x] 3.1 Add main and context page state with the fixed bottom row. Route existing screen and mode actions through the normal action path. Test screen changes and context pages.
- [x] 3.2 Give a visible question priority. Test immediate P7/P8 answers, an action and answer before redraw, destructive target checks, and PLAY/STOP that leave the question intact.
- [x] 3.3 Test text, picker, Help and learn priority. Run required changed-crate checks and commit this working step.

## 4. Context buttons and transport

- [x] 4.1 Add start-only, pause, continue and second-STOP reset actions. Keep Space, preview toggles and manually played tails unchanged. Test keyboard and raw MIDI paths through their real dispatch routes.
- [x] 4.2 Test A1 recording, project switch, queued commands and context preview. Render the pause/continue scenario to `renders/92-command-panel.wav` with finite, clipping, silence, timing and click checks.
- [x] 4.3 Run the 32-voice callback benchmark and required changed-crate checks before the audio-thread commit. Record any failure without changing the exit gate.

## 5. Drawing, help and owner checks

- [x] 5.1 Draw the panel in the current pad-grid rectangle, with page and safety feedback. Add F1 roles and P7/P8 question hints. Keep unavailable Redo and Panic dim.
- [x] 5.2 Add 80 by 26 and 120 by 40 catalogue cases with image overlay on and off. Compare the catalogue with the PR base. Add owner keyboard and SMC-PAD checks to `docs/LIVE-CHECKS.md` and update `docs/USER-STORIES.md`.
- [x] 5.3 Run format, Clippy, `task test:changed`, `task screens` and text checks. Review the full diff and commit the working step.

## 6. Final validation

- [x] 6.1 Run `task test:fast`, `task names`, `task specs:check`, STE lint and `git diff --check`. Report each required check and failure.
- [ ] 6.2 Publish a PR to `master` with separate keyboard and controller steps and the named WAV. Wait for green CI before merge. Keep physical control and listening results open until the owner checks them.

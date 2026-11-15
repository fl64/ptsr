# Design

The owner requested native SMC-PAD work on 2026-11-15, before the remaining sound and screen phases. Change 88 starts that work. The controller layout must make the owner's full session clear and practical. Later changes will test the hardware and the complete paths.

## Input contract

An open question or picker owns confirmation. A new press of its effective Enter key or learned `ui.confirm` button gives one confirm action to that owner. A release does not confirm. The current screen below the owner does not run its Enter action.

The question keeps its stored target and checks it again before a destructive change. Existing question-specific actions still answer their questions. Esc still cancels. A held pad release still reaches its original voice. Text input and MIDI learning keep priority over confirmation.

The folder picker selects its current folder. The project picker opens its selected project. If the owner already typed a new project name, confirmation uses that name. This change does not add controller navigation or controller text entry to the project picker.

## Routes

Add `Action::Confirm` with the public name `ui.confirm` in the GLOBAL button group. Add Enter to the shipped global keymap. Keep owner keymap overrides and saved MIDI bindings. In a question or picker, resolve the confirm route before edit and screen actions. A connected controller's learned GLOBAL binding can answer there even if the same control has a screen role.

Use the existing question and picker answer functions to apply the result. Do not create a second path that skips target checks, save guards, or the current action order. Update question help from the actions that production input can use. Do not show a controller answer from a profile template that is not connected.

## Edge cases

| Situation | Expected result | Status | Check |
| --- | --- | --- | --- |
| A controller press and release arrive while a question is open | The press confirms once. The release does nothing. | Support now | Raw MIDI sequence |
| A screen action shares the learned confirm control | The question or picker receives confirm. The screen action does not run. | Support now | Raw MIDI sequence on two screens |
| The question target changes before confirmation | The existing target check asks again or refuses the stale target. | Preserve existing behavior | Copy and file replacement tests |
| A Gate pad starts before the question and ends during it | Its original voice receives release. The question stays open. | Preserve existing behavior | Keyboard and raw MIDI sequences |
| A controller disconnects before the owner releases its button | Disconnect does not create a confirm press. | Support now | Disconnect sequence |
| MIDI learn or a typed name owns input | Learn stays pending. Typed text stays text. | Preserve existing behavior | Learn and picker tests |

## Rejected alternatives

Separate learned answers for every question would keep the owner dependent on per-screen setup. A direct application call from the MIDI handler would bypass current question and picker guards. Use one logical action through production dispatch.

## Open questions

- If an owner keymap removes Enter, should the question still accept it? Default: no. Explicit owner bindings keep priority. The owner can use another effective key or a learned controller button.

## Verification

Run the existing keyboard and raw MIDI harness through every current `Ask` variant and both pickers. Check the effective answer text at 80x26 and 120x40. Check the original X-Session profile and saved bindings. The change adds no audio render.

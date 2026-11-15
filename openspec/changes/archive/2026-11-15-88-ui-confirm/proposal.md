# Proposal

## Why

The keyboard can answer a question with Enter. A controller needs a different learned action for each question. This prevents the owner from completing some work without the keyboard.

## What Changes

- Add the `ui.confirm` action. Enter uses it while a question or picker is open.
- Let the owner learn one controller button for `ui.confirm` in GLOBAL.
- Give a question or picker priority over the action of the screen below it.
- Show the effective keyboard and connected controller controls that can answer the question.
- Keep the existing question answers and saved controller bindings working.

The change SHALL keep the sound engine, project format, and factory device profiles. It SHALL add no dependency.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `keyboard`: Enter confirms a question or picker through one action.
- `midi-learn`: a GLOBAL controller button can use that action.
- `screen-help`: questions name controls that can confirm them.

## Impact

The change affects input routing, question and picker answers, MIDI learning, and question help. It does not change audio processing.

## Out of scope

The SMC-PAD profile, command panel, button holds, and relative encoders remain separate changes.

## How to verify

### Keyboard

1. Open a replacement question for an occupied pad. Press Enter once. Check that only the named target changes.
2. Open a project picker. Select an existing project and press Enter. Check that the selected project opens.
3. Cancel a question and a picker with Esc. Check that neither action runs.

### Controller

1. Learn `ui.confirm` on a controller button in F12 MIDI. Open each question type and press the button. Check that each answer matches Enter.
2. Open a picker and press the button. Check that it selects the current item. Check that button release does nothing.
3. Hold a pad while a question opens. Release it. Check that the release stops only the original sound and does not confirm the question.

### Sound

This change adds no sound algorithm or render. Listen for the normal release of a held Gate pad after a question opens.

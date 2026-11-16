# Proposal

## Why

The measured SMC-PAD plays pads and changes some values. Its second pad bank has no command roles. The owner still needs the keyboard for screen and mode commands. Change 91 also left PLAY and STOP without starting roles because the existing toggle actions do not express their required behavior.

## What Changes

- Show a 4 by 4 command panel in the pad-grid area. Use the second PAD BANK layer in the same physical row order as the playing pads. The first second-layer hit only reveals a hidden panel.
- Give the panel a screen page and context pages for commands beyond 16 cells. Keep Back, Undo, Redo and Panic in fixed bottom cells. Redo and Panic stay visibly unavailable until change 93a. Keep LEFT and RIGHT on item selection.
- Let panel cells use logical actions through the normal input route. P7 cancels and P8 confirms a visible question. Destructive actions keep their questions.
- Give PLAY, STOP and RECORD screen-specific roles. PLAY starts or continues. STOP stops and a second tap returns to the start. PLAY and STOP never answer a question.
- Show panel state, active cells, pages, button roles and input results at both supported terminal sizes. Keep normal keyboard and X-Session actions.

This change SHALL NOT assign a host SHIFT action without a measured MIDI message. It SHALL NOT add pad lights, new project-file fields, Redo, Panic, or the complete controller-reach audit of change 93.

## Capabilities

### New Capabilities

- `command-panel`: panel entry, pages, cells, fixed controls and question handling.

### Modified Capabilities

- `input-devices`: SMC-PAD button roles and ordered second-layer routing.
- `screen-layout`: panel grid, badge, pages and controller feedback.
- `screen-help`: current controller commands and question answers.
- `transport`: separate start, stop and return-to-start actions.

## Impact

The change affects panel configuration, logical actions, MIDI dispatch, transport state, drawing, help and tests. Pause and continue change audio-thread clock behavior. Existing Space transport controls and owner assignments keep their current paths.

## How to verify

### Keyboard

1. Start and stop playback with the current keys. Open Files, Chop, Pattern, FX and Settings. Check that each key keeps its action.
2. Open a replacement question. Cancel and confirm it with the current keys. Check Undo in a disposable project.

### Controller

1. Start playback. Press PAD BANK and hit a second-layer pad once. Check that the panel appears, no cell acts and playback continues.
2. Hit screen and command cells. Change pages. Return to the first layer and hit a playing pad. Check the matching physical rows and uninterrupted sound.
3. Open a question. Use physical P7 to cancel and P8 to confirm. Check that PLAY and STOP do not answer it.
4. Tap STOP twice, then PLAY. Check pause, return to start and continue. Check Undo in a disposable project. Check that Redo and Panic are visibly unavailable.

### Sound

Listen to `renders/92-command-panel.wav` for the A1 pause, continue and return to start. Listen to an existing Gate pad while the panel opens and closes. The panel alone must not change playback.

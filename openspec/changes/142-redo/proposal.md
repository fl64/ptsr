## Why

An owner can undo an edit and hear the previous result. The owner needs a way to return to the result just undone.

## What Changes

- Press Ctrl+Y or use P3 Redo to restore an undone edit.
- Restore pad and FX edits, A1 rows and takes, Chop edits, and Assign choices.
- Keep Redo in the current edit context. A new undoable edit or a project change clears the redo history.
- Show the restored edit or the reason Redo cannot act. In Assign, use P3 Menu, then P3 Redo. Use P1 to return to the target view.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- editing: replay completed Undo steps.
- chopping: use Undo and Redo in the active Assign plan.
- keyboard: bind Ctrl+Y and respect owner bindings.
- command-panel: show available Redo on P3.
- screen-help: explain Redo and the Assign route.

## How to check

### Keyboard

1. Change a pad level. Press Ctrl+Z, then Ctrl+Y. Check the level and the notice.
2. Repeat with an A1 pass or one step edit. Play A1 again.
3. Repeat after a cut or an Assign choice. After Undo, make a new edit and press Ctrl+Y.

### SMC-PAD

1. Change a pad value. Hold PAD BANK. Press P2, then P3.
2. In Assign, select a target. Hold PAD BANK and press P2. Press P3 Menu, then P3 Redo.
3. Press P1 to return to the targets. Check the restored plan.

### Screens

Check `renders/screens/pads-help-80x26.ansi` and
`renders/screens/pads-help-120x40.ansi` for the keyboard and panel roles.
Check `renders/screens/pads-panel-main-80x26.ansi` and
`renders/screens/pads-panel-main-120x40.ansi` for the dim Redo cell when no edit waits.

### Sound

Listen to `renders/142-redo.wav`. Hear the edited result, the result after Undo, and the same edit restored by Redo.

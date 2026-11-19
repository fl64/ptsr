# Proposal

## Why

After `Save failed`, Exit can close ptsr with unsaved edits.
In a disposable project, Exit closed the app with +2 dB on screen and +1 dB in the file.
The owner needs to keep the work and see how to save it.

## What Changes

- Before Exit closes ptsr, save pending work in the current project or scratch session.
- If a save fails, keep the session open and show the session name and the reason.
- Show `Retry Save` and `Back` with the controls for each action.
- Use one manual save for each retry. After success, show `Session saved` and return to the session.
- Keep the completed A1 take and its Undo step if Exit cannot save it.
- Let the owner start a new A1 take after recovery.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `projects` covers safe exit, save recovery and A1 capture at the exit boundary.

## Impact

The change covers exit requests within ptsr and the recovery question.
It uses the current session file, save rules and keyboard and SMC-PAD question controls.
The owner approved the specification step on 2026-11-19.

## How to verify

### Keyboard

1. Open a disposable project. Load a sample onto a pad. Press Ctrl+S.
2. Move the project folder to a temporary name. Change a pad value. Press Ctrl+S.
3. Press Ctrl+Q. Verify the session name, the reason, `Retry Save` and `Back`.
4. Press Esc. Verify that the edit remains available.
5. Restore the folder. Press Ctrl+Q. Press Enter on `Retry Save`.
6. Verify `Session saved`. Press Ctrl+Q. Open the project again and verify the saved value.

### SMC-PAD

1. Open a disposable project. Save from the screens view of the command panel.
2. Move its folder to a temporary name. Change a pad value. Select Save again.
3. Select Exit from the screens view. Press the pad that the question names for Exit.
4. Press the shown Back pad. Verify that the edit remains available.
5. Restore the folder. Select Exit again. Press the pad that the question names for Exit.
6. Press the shown Retry Save pad. Verify `Session saved`.
7. Exit again. Open the project and verify the saved value.

Compare `renders/screens/pads-exit-recovery-80x26.ansi` and
`renders/screens/pads-exit-recovery-120x40.ansi`.
Use an A1 take with a held Gate note in the automated exit test.
Renders: none. The change governs saving and exit.

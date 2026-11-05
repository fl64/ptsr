# Visible modes and messages

## Why

The owner needs to see what a pad press does before pressing it.
Existing screens spread modes, recording, values and errors across different places.
A later message can replace an error. There is no message history.

Save safety and the report-only callback diagnostics are complete.
This change adds visual feedback for existing actions, not new sound controls.

## What Changes

- Show shared title badges, pad corner marks and a grid frame that identify current modes.
- Use yellow for modes where pads do more than ordinary playback, including marking, assignment, file placement, copy, swap and learning.
- Classify messages as Done, Info, Warn or Error. Protect higher-priority messages and keep the latest 50 in F1 Messages.
- Show changed values in the same place on every screen.
- Show recording, connected inputs, the last active input, audio and project in a shared status line.
- Show existing answer controls in questions. Do not advertise future controller buttons.
- Add an offline screen catalogue at 120×40 and 80×26, with graphics-overlay tests.
- Move screen-data preparation into its own module before adding behavior.

## Capabilities

### New Capabilities

None. These changes extend existing screen capabilities.

### Modified Capabilities

- `screen-layout`: title badges, a shared value popup, status segments, overlay safety and the screen catalogue.
- `screen-help`: message levels, lifetime, queue, history and truthful question controls.
- `pad-playback`: mode corner marks, yellow grid frames and compatibility with existing playback indicators.

## Impact

Application screen preparation, message producers, input-source metadata and terminal drawing need changes.
The preparation move is a separate behavior-neutral step.
Shared scenarios use keyboard and raw MIDI through production profiles and dispatch.
Notice timing uses controlled clocks. Terminal tests cover both catalogue sizes.

No new dependency, sound processing, factory binding or saved-file format is required.
Ordinary keyboard controls and existing input priority stay unchanged.
The F1 page controls are local to the open help overlay.
Manual save success keeps its five-second notice. Persistent save warnings stay independent from notices.

Out of scope: `ui.confirm` (88), button holds (89), relative encoders (90), the measured SMC-PAD profile (91), command panels and action echo (92).
Transport, patterns, pad lights and callback performance correction are also out of scope.
Hardware behavior and listening remain unverified.

## How to verify

### Keyboard

1. Load a sample onto a pad in a disposable project.
2. Press M on F2. Hit pads to mark them. Verify the badge, yellow frame and unchanged pad sound.
3. Press M again. Verify the marked count. Press Esc to clear marks.
4. Take a file on F6, or take a pad with Ctrl+C or Ctrl+X on F2. Verify the distinct mode labels.
5. On F3, press Y and P. Verify planned slice numbers. Press Esc.
6. Open pad edit with Enter. Change a value with J/L. Verify its object, name and value in the popup.
7. Repeat the refused-save procedure in `docs/LIVE-CHECKS.md`, section 86b.
8. Verify that an informational message does not remove the error. Open F1 Messages to find both results.
9. Restore write access. Press Ctrl+S. Verify `Session saved` for five seconds and removal of `Save failed`.

### Controller

1. Use an existing working profile or learn unused controls in F12. Do not assume the SMC-PAD template is operational.
2. Learn `pad.mark_toggle`, `pad.copy`, `pad.take` and `session.save` buttons, plus a `pad.volume` knob.
3. Repeat the marking, placement, value and save procedures through those controls and MIDI pad notes.
4. Alternate keyboard and controller actions. Verify the input indicator and unchanged action results.
5. For questions without an existing controller answer, use the keyboard answers in the question. Controller-only confirmation belongs to 88.

### Offline screens and sound

Run the planned `task screens` after implementation.
Inspect `renders/screens/pads-mark-120x40.ansi`, `pads-copy-80x26.ansi`, `chop-assign-80x26.ansi` and `help-messages-80x26.ansi`.
The catalogue includes every registered screen and applicable existing mode, not only these examples.
Sound processing stays unchanged. No new sound render is required.
Existing pad, recording, project-isolation and sound comparison tests remain required.

# Proposal

## Why

The owner can play the prepared A1 rhythm from change 96, but cannot record a rhythm with the pads.
This change adds live recording without a TOML edit.

## What Changes

- Ctrl+R starts or ends pattern recording on every current screen without changing screens.
- With no A1, create an empty two-bar A1 at 1/16 and 100 percent strength.
- From stopped, play one bar of count-in before recording. During playback, start recording at the accepted audio boundary.
- Add new hits to existing hits. One recording pass makes one Undo step.
- Keep manual hits immediate. Apply quantize on later loops without changing the raw take or producing a second immediate hit.
- Add compact controls for bars, quantize and strength. Add silent pad-held erasure during recording.
- Show count-in, `REC A1`, erasure, loop position and current pattern settings.
- Clarify existing Pitch Gate feedback: Off, On for A1 only, or On but inapplicable with a text reason.
  Keep manual hits unchanged. Do not add a top badge or imply an active cut from the saved switch.
- Add learnable MIDI buttons and absolute parameter controls through existing input routes.
- **BREAKING:** shipped Ctrl+R starts pattern recording instead of showing the microphone move notice. Explicit owner bindings still win.
- **BREAKING:** during pattern count-in, recording or pattern controls, Chop pad keys play real assigned pads, not unassigned slice previews.
- Keep Alt+R microphone and Ctrl+T output recording separate. Refuse conflicting recording requests with an explanation.

## Capabilities

### New Capabilities

None independent of change 96. The `patterns` capability originates in that active change.
The 97 delta adds recording requirements to that capability without copying its playback requirements.

### Modified Capabilities

- `patterns`: recording passes, count-in, gate capture, erasure, Undo and pattern settings.
- `keyboard`: recording keys, owner overrides and input priority.
- `midi-learn`: recording commands and pattern parameter learning.
- `screen-layout`: count-in, recording, erasure and Pitch Gate applicability feedback without a step editor.
- `screen-help`: effective recording controls, next steps and the Pitch Gate manual-play exclusion.
- `recording`: separation from microphone and output capture.
- `projects`: complete recording passes, saving and project boundaries.
- `chopping`: real-pad input during pattern work and continued rhythm on entry during recording.

## Impact

The implementation affects the engine clock, sequencer and bounded recording feedback.
It also affects shared input routing, App pattern state, Undo, saving, MIDI learning and terminal views.
Existing format 5 stores the raw hits and settings. Add no saved recording state or format migration.
Add no dependency. Preserve pad sound, source files, assignments, mute groups and existing callback tests.

Implementation starts from local change 96 on branch `97-pattern-record`.
The owner requested immediate continuation before publication of 96. Keep separate atomic commits.
This exception does not authorize publication, archive, merge or device access.
The earlier 96 Chop solo-preview question and historical callback maximum failures remain open.

## Out of scope

- The F5 step grid, note editing and pad rows belong to 98.
- Multiple patterns, redo, controller hold recognition and factory SMC-PAD mappings remain separate work.
- Do not change owner projects, add migration work, change audio algorithms or diagnose the deferred Chop click.
- Do not claim physical-controller comfort, listening acceptance or a live terminal pass from offline tests.

## How to verify

These procedures apply after implementation. They are not current verification results.

### Keyboard

1. Open a disposable project with sounds on pads.
2. Press Ctrl+R while stopped.
3. Hear four count-in beats. Verify that `REC A1` starts at the first pattern beat.
4. Play a rhythm. Press Ctrl+R again.
5. Hear the rhythm continue. Press Ctrl+R to add another pass.
6. End that pass. Press Ctrl+Z on Pads to undo only that pass and its pattern-setting changes.
7. Open pattern controls with Alt+N. Compare Off, 1/16 and a triplet grid.
8. During recording, enable Alt+E. Hold one pad through a beat, then disable Alt+E.
9. Verify silent erasure of that pad only. End recording and undo the pass.
10. Save and reopen. Verify the rhythm, settings and stopped transport.
11. On Pads, toggle Pitch Gate with Alt+G. Verify Off, A1-only and inapplicable-source or pitch feedback.
12. Play a pad manually with Pitch Gate On and Off. Verify unchanged sound and the visible manual-play explanation.

### Controller

1. In F12 MIDI, learn separate buttons for pattern recording, pattern controls, erasure and pattern Undo.
2. Press Enter to save the bindings. Return to Pads.
3. Run the keyboard procedure with those buttons and controller pads.
4. Repeat recording start and end on Files, Chop, FX and Settings.
5. Verify that releases do not toggle commands again or change the recording target.
6. Learn a PADS Pitch Gate button in F12. Compare its labels and popup with the keyboard.
7. Play manual pads while A1 runs. Verify that the hint still excludes manual cuts.

These are learned controls, not measured SMC-PAD factory support.

### Sound and terminal views

Listen to the future local file `renders/97-pattern-record.wav`.
Compare the raw take, quantized repeat, added pass, erased beat and Undo result.
Listen for doubled hits, missing beat starts, incorrect gates and changes to attacks.
Inspect future count-in, recording, erasure and controls catalogue states at 80×26 and 120×40.
Include Pitch Gate Off, A1-only and inapplicable states with source and pitch reasons.
This indicator clarification changes no sound. Use the existing `renders/96-pitch-gate.wav` for the On, Off and manual comparison.
No new Pitch Gate listening file is necessary for this text-only clarification.

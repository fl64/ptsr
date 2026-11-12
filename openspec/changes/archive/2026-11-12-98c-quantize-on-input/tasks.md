# Tasks

## 1. Capture and playback

- [x] 1.1 Quantize each accepted row at capture with the current Quantize and Strength. Keep the Gate duration.
- [x] 1.2 Publish the pattern for playback with quantize Off.
- [x] 1.3 Apply the replacement rule to saved ticks, in one pass and across passes.

## 2. Old projects

- [x] 2.1 Convert the owner's projects once by a script outside the program, with a backup. Check that each A1 compiles to the same onsets.

## 3. Text

- [x] 3.1 Show `next hits` beside Quantize and Strength in pattern controls. Update F1 and the README.

## 4. Verification

- [x] 4.1 Cover every scenario with keyboard and raw MIDI tests.
- [x] 4.2 Render `renders/98c-quantize-on-input.wav` with checks for timing, clipping and clicks. Run `task check` and `task build`.
- [ ] 4.3 Owner check of the steps in the proposal.

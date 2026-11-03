## Why

Keyboard and MIDI use separate rules to decide what a pad hit does. A controller
command can also reach the screen too late: taking a file and then hitting a pad
before the next screen update can play the old sound instead of choosing its target.

## What Changes

- Put the existing pad rules in one place for keyboard and MIDI. Give the input
  state a name and publish it to MIDI as one complete value, not five separate values.
- Keep the current keys, controller assignments, sounding marks, Chop listen and
  assignment, questions, undo, velocity and original pad releases.
- Separately fix the order of controller commands and following pad hits. A hit after
  taking a file must select its target, without playing the old sound. When a preceding
  command needs the screen, a following hit can wait for that command. Normal performance
  hits with no such work pending must still go straight to audio, without a second hit.
- Keep text, pickers and F12 learn at their current input boundaries. Do not make the
  roadmap's proposed priority order into a new behavior rule.
- Move the input dispatch into its own file in a separate move-only step.

## Capabilities

### New Capabilities

None. This change adds no user mode or command.

### Modified Capabilities

- `input-devices`: specify the pending-command exception to direct MIDI playback,
  ordered target hits and safe releases for hits that wait for the UI. The shared
  input model and code move do not change other requirements.

## Impact

Implementation is limited to `crates/input/src/lib.rs` and
`crates/app/src/{main,midi,input_tests,dispatch}.rs`, with tests beside those areas.
The new `dispatch.rs` holds existing input methods; it is not a new dispatch path.
No dependency, audio callback, profile, keymap or saved-data format changes are needed.
Owner checks and current-status documents will be updated in a separate documentation
step. The baseline is `f52cf00`, after the accepted 84 and 84a archives.

The owner approved preparation of 85, then accepted the proposed narrow plan after
its benefits were explained. Work continues on `plan/controller-first-patterns` as
stated in that plan. Push and PR still need a separate request. These artifacts need
review before implementation. The SMC-PAD has not been measured and the owner has no
controller at present; recorded MIDI is not a hardware check.

## Out of scope

New keys, modes or profiles; SMC-PAD setup; visible badges or debug-only UI changes;
confirmation redesign; relative knobs; transport; audio optimization; callback timing
investigation; text, picker or F12-learn priority repairs. Do not change Space, M, the
owner's Chop cursor, or the playback line and badges from 84. Do not connect to the
owner's sound graph, change the owner's configuration or claim hardware acceptance.

## How to verify

### Keyboard only

1. Play and release pads on Pads, in pad edit and with marking on. Marks must still
   sound. Turn banks while a gate is held, then release its original key.
2. In Files, select a file, press Enter to take it, then hit an occupied pad. It must
   select the target and ask about replacement without a pad hit. Esc must keep the
   old sound and the file in hand. Copy, swap and their undo must work as before.
3. On Chop, check listen before and after linking slices, slice edit and by-hand
   assignment. A question and F10 must block new hits; an older held pad must release.
4. Text input, pickers and F12 learn must keep their present key behavior. This change
   does not repair differences between those keyboard and MIDI boundaries.

### Controller only, when hardware is available

1. Use the existing pad notes 36–51 or an explicitly learned pad control. In Files,
   take a file with a bound command and hit an occupied target immediately. Only the
   replacement question must result; the old pad must not sound. Do not reset a
   profile or assume that the incomplete SMC-PAD template is a working profile.
2. Cancel the hand state, then play ordinary pads. They must still sound once without
   waiting for a screen refresh. Check soft and strong hits.
3. Hold and release a gate across bank, screen, question and F10 changes. Repeat with
   a learned pad button and, where available, a second device. Only the original pad
   must release. Check the existing safe X-Session navigation and knob roles.

### Automated checks and listening

Use the real keyboard and raw-MIDI driver from 84 with explicit frame fences and both
real offline audio queues. Tests must force the command/hit order without sleeps,
including a release or disconnect before the UI handles a deferred hit. Run the full
`task check`, strict OpenSpec validation and `task build` before completion.

Keep `renders/84-input-parity.wav` unchanged. Add `renders/85-input-modes.wav` with a
silent take-file/target segment, a normal direct hit and a deferred gate hit followed
by its release. Tests check finite samples, no clipping, intended silence, one hit
and release timing within one sample of the expected audio block. Listen for no old
sound at the target, no doubled hit and no stuck gate. No listening result is claimed
by an automated test. Physical-controller checks remain unverified until hardware
is available; archive requires explicit owner acceptance.

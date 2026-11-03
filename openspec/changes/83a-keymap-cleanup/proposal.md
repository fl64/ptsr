## Why

The same keys change banks on Pads but values on FX. Microphone recording and Chop use
keys needed later for pattern recording and redo. Clean up these keys before change 84
uses the keyboard as the reference for controller behavior. Tell the owner where an
unbound old key moved; do not change their own bindings.

## What Changes

- On Pads, J/L and Left/Right change the selected value, with edit open or closed.
  U/O turn banks. PageUp/PageDown, Ctrl+1…8 and controller bank actions stay.
- Move microphone hold from Ctrl+R to Alt+R. Ctrl+T still records the output. Without
  key releases, Alt+R toggles microphone recording, as the old key did.
- On Chop, Ctrl+G normalizes, H lowers the hit threshold and Shift+H raises it one step.
  Tab uses the named action `chop.assign_order`. Old `edit.param_next` bindings still
  switch the assign order while assigning.
- Add `pad.copy` on Ctrl+C on Pads: take a copy in hand, then hit a target in any bank.
  Keep the source, ask before replacing an occupied target, and support one-step undo.
- Explain moved keys with the current flash. Warn at start when an owner binding hides
  a new default; the owner's binding wins. Add the three new actions to both MIDI learn
  views. Keep the X-Session profile and existing learned bindings unchanged.
- Test key consistency, both keyboard layouts, input with and without key releases,
  assignment, copying, help and the existing recording safeguards.

## Owner decisions

On 2026-11-03 the owner confirmed option A: J/L change values on Pads, U/O turn banks.
The owner also requested this work in `plan/controller-first-patterns`, not a new branch.
This is an exception to the usual branch-per-change rule. Keep one active change and
small, separate commits. The approved step now is specifications only; code is not part
of this commit.

## Capabilities

### Modified Capabilities

- `keyboard`: consistent key roles, reserved chords, moved-key notices and owner warnings.
- `recording`: Alt+R microphone hold or toggle; output and sound-graph safeguards stay.
- `chopping`: moved keys, reverse hit threshold and the named assign-order action.
- `editing`: value keys on Pads with edit closed, without changing Chop navigation.
- `pad-playback`: U/O banks, value keys and undo of a pad copy.
- `pad-assignment`: copy a pad from either input without changing its source.
- `screen-help`: current keys, Shift+H and copy-in-hand instructions.
- `midi-learn`: learn the three new actions in their screen groups.

## Impact

Later implementation touches the shipped keymap, input resolution, action list, App,
pad state helpers, Chop, help, tests and owner documents. It needs no new dependency,
audio algorithm, project format or user-file migration. Main specs are not changed by
this specifications-only step; deltas stay active until implementation and acceptance.

## Out of scope

Patterns, transport, panic, redo, the input refactor, the SMC-PAD profile, the command
panel and the new notice model. Do not bind future actions to placeholders. Do not
rewrite the owner's keymap, learned MIDI controls or files in Downloads. Change 84
remains deferred. No live audio run is needed to prepare this specification.

## How to verify

After implementation, use the default keys, then repeat key checks with an owner override:

1. On Pads, press I/K to pick volume, pitch and envelope parameters. J/L and the arrows
   change values before Enter and during edit. U/O turn banks in both states.
2. Hold Alt+R, speak and release: the microphone recording is in hand. On a terminal
   without releases, press Alt+R twice instead. Ctrl+R only says `microphone moved to
   Alt+R`. Ctrl+T still records output. Neither recording interrupts the other.
3. On Chop, use Y for equal parts, H for hit cuts, Shift+H for fewer cuts and Ctrl+G
   for normalize. Unbound Ctrl+Y and Ctrl+N say where the commands moved. Tab changes
   assign order only during assignment. An old learned `edit.param_next` still does it.
4. On Pads, press Ctrl+C and hit an empty pad in another bank. Source and copy play the
   same with their full settings; the copy's name ends in ` 2`. Try an occupied target:
   Enter replaces, Esc keeps it and leaves the copy ready for another target. Ctrl+Z
   restores the target. Esc or a hit on the source cancels a pending copy.
5. Repeat the copy target with a controller pad. On F10 and the F12 MIDI tab, learn
   `pad.copy`, `chop.assign_order` and `chop.hits_back` to buttons and test them.
   Do not reset or change the physical X-Session defaults for this check.
6. Open F1 and check the bottom help on Pads, Chop, during Shift and with a copy in hand.
   Repeat in a terminal without the extended keyboard protocol and in Russian layout.
   Set an owner override for a moved or new key: it must win, with a warning only when
   it hides a changed default. Typed text must stay text and pads must stay silent.
7. Listen to `renders/83a-pad-copy.wav` after implementation. It must compare the source
   and copy, then the restored target after undo, with no clicks or clipping. This WAV
   is a planned acceptance artifact; it is not made by the specifications-only step.

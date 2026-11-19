# Proposal

## Why

The owner cannot work with the SMC-PAD without a memo. On 2026-11-19 a live audit
walked 12 scenario groups through the real SMC-PAD profile and a virtual port.
The groups were file load, pad edit, pad FX, the FX screen, copy, clear, undo,
chop, assign, A1, settings, learn, help, save, open and exit. The audit
found seven causes that repeat on all screens:

1. One pad has three numbers: keyboard keys on the grid, device numbers on the
   panel and top-down numbers in Help. Rows show 1, 2, 3, but knobs 7, 5, 3 turn them.
2. The screen shows wrong data. The panel title hides the screen name and
   overlaps badges. Cells behind a question show other actions. The target
   title names the wrong pads.
3. Hints and mode texts name only keyboard keys.
4. Yes and No use different pads: P11/P12 in questions, P1/P4 in Assign, P11/P12
   as Help tabs.
5. The target view shows `pad` or `used` and no pad name or planned slice.
   Only 12 of 16 pads are targets.
6. Some actions do nothing and say nothing.
7. Some labels do not tell the result.

The main product goal in `AGENTS.md` asks for a clear screen at each step and the
same control for the same action. This change applies that goal to the current
actions. It adds no new sampler feature.

## What Changes

- The SMC-PAD uses one pad number everywhere: the number printed on the device.
  The grid, the panel, the side panel, messages and Help use it with an SMC-PAD
  connection. The keyboard key shows in the pad corner.
- The bottom row of the panel is the control row on every view. P1 is No, Back
  or Cancel. P2 is Undo. P3 is Redo. P4 is Yes, Apply or Done. Panic leaves the panel.
- Every question answers with P1 No and P4 Yes. Without a hold, these two pads
  answer and the other pads stay silent. Learn, Exit and the project list use the
  same pads.
- Help tabs move with LEFT and RIGHT. P11 and P12 have no special role in Help.
- Hints follow the device of the last input. After an SMC-PAD input, hints and mode
  texts name SMC-PAD controls. After a key, they name keys.
- A mode line shows the active mode, the item in hand or the running operation,
  the next step and the cancel control. It stays visible after PAD BANK release.
- Placing an item on a pad uses the playing layer: copy, swap, file, recording
  and Assign. Without a hold, all 16 pads are targets. The grid shows each pad
  name, its state and the planned slice. A hold shows the commands: P1 Cancel,
  P2 Undo, P3 Menu, P4 Apply where Assign needs it.
- The title keeps the screen name while the panel shows. Badges do not overlap.
  Cells behind a question show only the two answers.
- A row in a page names the knob that turns it. The knob strip and the row use
  the same parameter name.
- A refused panel action says why. A result names its target pads.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `command-panel`: the control row, question pads, the target view and the panel title.
- `controller-reach`: Learn, Exit, Help and picker answers, and refusal messages.
- `screen-layout`: device pad numbers, knob numbers on rows and the mode line.
- `screen-help`: hints by the last input device.

## Impact

The change affects panel drawing, question routing for the SMC-PAD, the target
mode, hint text, pad labels, Help and the route documents. Keyboard keys and
the keymap stay unchanged. The saved config and project formats stay unchanged,
so no migration is necessary. The X-Session profile keeps its routes. Sound
stays unchanged. No new dependency.

Owner checks, controller:

1. Hit the pad printed 1. Check that the screen says pad 1.
2. Copy pad 1 onto an occupied pad. Answer with P1, then again with P4.
3. Open Exit and answer with P1. Open Help and change tabs with LEFT and RIGHT.
4. Start Assign. Release PAD BANK and check that all 16 pads show their names
   and planned slices. Place slices on pads 1 and 16. Hold PAD BANK and press P4.
5. Load a file with Take. Check that the mode line says what is in hand, the next
   step and how to cancel, with PAD BANK released.
6. In pad edit, turn knob 7 and check that the volume row shows 7.

Owner checks, keyboard: press a key after an SMC-PAD input. Check that the hints
name keys again. Check that Enter and Esc still answer questions.

Renders: none, because sound stays unchanged.

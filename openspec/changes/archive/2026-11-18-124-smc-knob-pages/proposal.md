# Proposal

## Why

A knob can have a different role after a screen change. Some paired controls sit on
different rows. The encoders have no detents, so a turn can skip a page. KNOB BANK
sends no event on its own. The owner needs controls that are easy to find by touch.

## What Changes

- Use LEFT and RIGHT for the main list of the current screen or mode.
- Give bank 1 knob 1 the position. Put current page values on knobs 2–7.
  Place related values in the same physical row, with the lower or earlier value first.
- Give bank 2 the selected pad. Put master volume on knob 8 in both banks.
- Move tempo to the Pattern transport page. Reserve its tap cell for Tap Tempo.
- Give Chop separate pairs for slice bounds and region bounds. Make slow zoom turns finer.
- Replace shipped panel value and parameter steps with direct controls.
  Keep the keyboard and learn actions. Use panel cells for panel page moves.
- Show both banks in physical order. Mark the bank of the last knob input.
  Name the current page and the role of LEFT and RIGHT.
- Show the change to an existing saved SMC-PAD map before the owner applies it.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `controller-reach`: change the SMC-PAD list and knob routes.
- `command-panel`: change page navigation and remove replaced cells.
- `input-devices`: preserve saved owner bindings during the layout update.
- `screen-layout`: show the current physical roles and the last knob input.

## Impact

This change affects the SMC-PAD profile, MIDI routing, the command panel and screen
hints. It uses existing sound controls and keeps the current project format.
The later panel change covers PAD BANK defaults, target flows, labels and device
warnings. Pan and Note Repeat follow their own changes.

## How to verify

Compare these screen catalogue entries at both supported sizes:

- `pads-edit-env` and `pads-panel-main`.
- `chop-edit-bounds` and `chop-panel-context`.
- `pads-pattern-controls` and `fx-fx-2`.
- `files-normal`, `settings-panel-main` and `chop-panel-question`.

### Keyboard

1. On Pads, press Alt+Left and Alt+Right. Verify that they select a pad silently.
2. Open pad edit. Change pages, parameters and values with the current keys.
3. On Chop, move both slice bounds with the current keys.
4. Step the threshold with Ctrl+Alt+Up and Ctrl+Alt+Down. Verify both limits.
5. Open a question. Use its cancel key and verify the original target.

### Controller

1. On Pads, press LEFT and RIGHT. Verify the pad bank.
2. Hit a pad. Turn bank 1 knob 1. Verify that the next pad in the row stays silent.
3. Open pad edit. Change its page with LEFT or RIGHT.
4. On ENV, turn knobs 3 and 4. Verify attack and decay.
5. Open Chop. Turn knob 2 slowly. Verify that the wave grows in small steps.
6. Turn knobs 3 and 4. Verify the two slice bounds.
7. Turn knobs 5 and 6. Verify the two region bounds.
8. Turn knob 8 in each bank. Verify that both turns change master volume.
9. Open Pattern, Files, FX and Settings. Compare LEFT and RIGHT with the help line.
10. Open a question while a Gate pad sounds. Use P7 to cancel. Release the pad.
11. Verify both banks at 80 by 26 and 120 by 40. Turn one knob in each bank.
12. With a saved old map, open F12 MIDI. Select `SMC-PAD layout review`.
13. Compare the old and proposed roles. Cancel one question with P7.
14. Select a change and accept it with P8. Restart. Verify the new role and a personal binding.

### Sound

This change introduces no sound algorithm. Use `renders/92-command-panel.wav` as the
transport reference. Listen to a held Gate during a page change. Listen for smooth
master changes in both banks and for each send on a pad with two active FX slots.

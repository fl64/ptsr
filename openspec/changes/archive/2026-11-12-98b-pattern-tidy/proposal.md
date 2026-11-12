# Proposal

## Why

The owner asked how to erase all hits of one pad or all of A1. Today only held erasure inside a recording pass can do it.
A first version used Alt+Backspace on the selected pad and command rows run by J/L. The owner found both unclear: the key works only outside a pass, and nothing shows the hits of a pad before the erasure.
Saved repeats from older passes show as `2` in the strip. The owner did not see what `2` means.
The `⇢` glyph on every one-shot pad reads as a position arrow.

## What Changes

- Alt+E (`pattern.erase`) also works on Pads outside a pass. Then a pad press erases all A1 hits of that pad, without sound. Each erasure is one Undo step. Pads show their hit counts in this mode.
- Inside a pass, Alt+E keeps the held erasure under the playhead. A pass start or end turns off erasure.
- Add a Clear A1 row to pattern controls (`pattern.clear`). Enter runs it, and it asks first.
- Make `pattern.clear` a learnable MIDI button. Post a notice with the count of each result.
- End the strip header with `2-9 = hits in a cell, one plays` while such a cell is visible.
- Show no mode glyph for one-shot pads. Gate pads keep `⇥`.

The owner approved this work on branch `97-pattern-record`, next to 97 and 98.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `patterns`: whole-pad erasure outside a pass and clear A1.
- `keyboard`: Alt+E and Esc for erasure, Enter on the Clear A1 row.
- `midi-learn`: learnable Clear A1 and erasure through MIDI pads.
- `screen-help`: the Clear A1 row, erasure help, F1 and result notices.
- `screen-layout`: hit counts in erasure and the multiple-hit legend.
- `pad-playback` and `editing`: no mode glyph for one-shot pads.

## Impact

App pattern edits, shared input defaults, pattern controls, help and the pad mode line.
No sound algorithm, callback work, dependency or saved-format change.

## Out of scope

- A merge command for repeats. Playback plays one hit per cell, and a pass end replaces repeats. A merge would erase hits for good and can change which velocity plays.
- The step editor (draft [98a](../../../docs/plans/98a-step-edit.md)).
- Default Pitch Gate values. The owner left them unchanged.
- SMC-PAD command panel cells. Change 92 adds the panel.

## How to check

1. Record A1 with pads Q and W. Stop the recording. On Pads, press Alt+E. Check `ERASE` and the hit counts on Q and W.
2. Press pad Q. Check that it makes no sound, its hits disappear and W hits stay. Press Ctrl+Z, then Esc.
3. Start playback, press Alt+E and press pad W. Check that W stops in the next loop without a click.
4. Press Alt+N. Select Clear A1 and press Enter. Answer keep with Esc, then repeat and answer yes. Press Ctrl+Z.
5. In F12 MIDI, learn `pattern.erase` and `pattern.clear` on buttons and repeat steps 1–4 with them and the controller pads.
6. Check that one-shot pads show no `⇢` and gate pads show `⇥`.

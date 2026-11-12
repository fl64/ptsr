# Design

## Decisions

- **One eraser.** Alt+E is the only erase key, as an Erase button on a hardware sampler. In a pass, a held pad erases under the playhead. Outside a pass, a pad press erases the whole pad. The owner rejected Alt+Backspace on the selected pad: it worked only outside a pass, and nothing showed the hits before the erasure.
- **Visible target.** Whole-pad erasure shows `ERASE` and the hit count on each pad. The owner sees what a press erases before the press.
- **Pads only, outside a pass.** The counts show only on Pads, so whole-pad erasure is a mode of Pads. A screen change ends it. Other screens refuse the toggle with a notice that names Pads. Inside a pass, erasure works on every screen, as in 97.
- **No mode change by itself.** A pass start turns off whole-pad erasure, and a pass end turns off held erasure. So held erasure never becomes whole-pad erasure without a new toggle.
- **Esc first leaves erasure.** Esc closes the innermost state. A second Esc keeps its 97 role.
- **Empty A1.** Without rows, the toggle refuses with a notice, so silent pads never mean an empty mode.
- **Playback.** Whole-pad erasure does not stop transport. A sounding voice of the pad finishes its release, so the erasure makes no click.
- **No key for Clear A1.** A whole-pattern loss from one key press costs too much, even with Undo. Clear A1 is a row of pattern controls and asks first, like Clear pads.
- **Enter runs a command row.** In a menu, Enter runs a command. The change keys only change values. Enter on a value row keeps closing controls.
- **No merge.** Playback plays one hit per pad and cell, and a pass end replaces repeats. So repeats are not audible. A merge would erase hits for good, would not undo a later quantize change, and could change which velocity plays. The legend says that one hit plays.
- **Undo.** Each command is one common-history Undo step with the pattern snapshot that pass Undo uses.
- **Refusal.** Clear A1 refuses while a recording pass is active or pending. The notice says to end the pass first. Erasure does not start during CountIn or Finishing.
- **Controller.** A learned `pattern.erase` button and the controller pads give the same flow. The SMC-PAD plan already has a latched Erase cell in the Pattern panel. STOP tap (`ui.back`) leaves erasure.
- **Archive order.** The `patterns` and `keyboard` deltas change 97 requirements, and the `screen-layout` delta changes 98 requirements. Archive 97 and 98 before 98b.
- **View and quantize apart.** The owner saw `grid 1/16` and hits on quarters: the header named the view, not the quantize. Now it names both. The file name left the strip: it did not fit, and the pad panel shows it.
- **Rows at start.** The owner wants to see the pattern at once. The splash keeps only the wave rows.
- **Controls take the column.** The owner saw the A1 panel over the top of the pad panel as one confusing menu. Pattern controls now take the whole right column in the pad panel style. While they are open, edit keys and knobs act only on A1. Knobs 1–3 turn Bars, Quantize and Strength, as the pad edit knobs turn its page. Pads still play, so the owner can hear the change.
- **One edit look.** The owner asked for one look and one behavior for every edit element. Pattern controls take the pad edit frame and its title with lit blocks. An audit of the other edit elements (FX, Settings, Chop slice edit) is a separate change.
- **Mode glyph.** One-shot is the default, so it needs no mark. Gate keeps `⇥`. The panel names the mode in words.
- **Legend.** The header legend sits at the end, so a narrow screen cuts the legend before the pad name.

## Open questions

- Pitch Gate default. The owner did not see its effect yet. Default: no change. A WAV comparison can follow.
- Whole-pad erasure on the future Pattern screen (98a or later). Default: it works there too, because that screen shows the hits.

# Design — 06d

## Palette

Eight hues spread around the color wheel, all at a similar soft lightness, so none reads as an alarm and all stay readable on the panel (`#11141A`):

| Bank | Color |
| --- | --- |
| 1 | `#D98880` coral |
| 2 | `#D9A066` amber |
| 3 | `#C9C46E` olive |
| 4 | `#86C47E` green |
| 5 | `#6FC2B4` teal |
| 6 | `#7FA3E0` blue |
| 7 | `#A88BE0` violet |
| 8 | `#D08CC4` orchid |

None of them is cyan `#33E1FF` or magenta `#FF5FAE`: those keep their meanings (selected/playing, attention), and the bank colors are less saturated than both.

## Where

`tui::BANK_COLORS`; `banks()` colors the bank row; `pad_cell` takes the bank color for an idle frame. `docs/ui/STYLE.md` lists the palette as the one allowed exception to "no other accents": the bank tints mark belonging, not state.

## Open questions

1. **Exact tones.** Picked by the agent; the owner adjusts them after seeing them.

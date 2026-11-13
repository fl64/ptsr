# Design

## Context

The current specifications require `empty`, a flat wave in an empty pad cell, `· all` for a whole-file source, and visible idle PLAY text. Change 122 replaces those text rules. It keeps the data and playback rules.

On 2026-11-13, the owner authorized this change after we named these conflicts. The owner did not report a separate hardware result for 122.

## Decisions

### Keep screen geometry

The idle PLAY text becomes blank inside its existing ten-cell slot. Active PLAY and LISTEN retain their width, positions, and rules. The clip row also stays in place when its label is hidden. This keeps other text from shifting.

### Keep full Pitch Gate state available

The compact Pitch Gate summary appears only when pad pitch differs from zero. The pad panel, popup, and F1 continue to show the saved state and any reason it cannot act. A zero-pitch pad does not lose access to its Pitch Gate setting.

### Use a session-relative clip time

The UI meter records the latest clipped block time on the UI thread. The label uses `CLIP +HH:MM:SS`, elapsed since program start. A clean hit clears the existing latch and hides the label. A restart resets the time. Tests use a controlled clock.

### Keep technical identifiers stable

`all`, action IDs, saved MIDI group IDs, project fields, and key bindings stay unchanged. Only their shown names change. The visible terms are `transport`, `pattern`, `pad`, `slice`, `sample`, `bank`, `send`, `FX 1`, and `FX 2`.

### Clip words without hiding required text

Long names lose complete words before `…` when a word fits the field. If one word is wider than the field, clip at grapheme boundaries. Safety warnings, key caps, numeric values, and units keep their current priority.

## Open questions

- Should the clip time use wall-clock time? Default: elapsed session time. It does not depend on clock changes.
- Should the compact Pitch Gate summary appear at zero pitch? Default: hide it there. The panel and F1 still show the saved state.
- Should 122 reassign unused body rows at 80x26? Default: leave geometry to `screen-layouts` so this wording change stays small.

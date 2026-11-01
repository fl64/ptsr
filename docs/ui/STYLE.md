# UI style (draft)

Cyberpunk: an almost black background, two neon accents with a meaning, everything else muted. The screen sketches in `docs/ui/` are a mood guideline, not pixel-exact. A terminal has one font size, no rounded corners and no semi-transparency.

## Colors

| Role | Color | Where |
| --- | --- | --- |
| Background | `#07080A` | whole screen |
| Panel | `#11141A` | pads, tiles |
| Frame | `#2A303A` | frames, inactive items |
| Text | `#C9D1D9` | main |
| Bright text | `#E6EAF0` | headings, values |
| Muted | `#8A929E` | labels, help |
| Cyan | `#33E1FF` | playing, selected, on |
| Magenta | `#FF5FAE` | cut markers, waiting, recording, warnings |
| White | `#F2F4F7` | hit flash, playhead |

No other accents. If you want to highlight something with a third color, the screen has too much on it.

The exception is the bank colors. Each of the 8 banks has its own muted tint. This tint colors the bank number and fill in the bank row, the pad frames and the key letters of the open bank (the open bank is bold in brackets). It shows membership, not state: the selected pad stays cyan, the pad in hand stays magenta, a hit is a white flash. Assign mode on the chop screen colors the bank row in its own way: the planned bank is magenta, an occupied bank is bright text, an empty bank is muted.

| Bank | Color |
| --- | --- |
| 1 | `#D98880` coral |
| 2 | `#D9A066` amber |
| 3 | `#C9C46E` olive |
| 4 | `#86C47E` green |
| 5 | `#6FC2B4` turquoise |
| 6 | `#7FA3E0` blue |
| 7 | `#A88BE0` violet |
| 8 | `#D08CC4` orchid |

## Rules

- The screen shows only what is necessary now. Details are in F1 or in pop-up values.
- The color shows the state at one look: cyan means sounds/selected, magenta means waits/cuts/attention.
- The screen name is an inverse badge at the top left (`F3 CHOP`).
- The help at the bottom: groups (`LISTEN`, `CUT`, `ENV`, …) → key button (light text on the gray frame color `#2A303A`) → action. The help shows a toggle state only if the screen does not show it anywhere else (the pad mode and pad volume are already in the pad panel).
- With no modifier held, the help shows only keys without modifiers. With Ctrl held: only the Ctrl combinations of the screen. With Shift held: only "pads at full velocity". In a terminal without the kitty protocol, the Ctrl combinations stay in the usual line.
- In the sketches `docs/ui/*.md`, a key in brackets `[F1]` means such a button.
- The hint line for the next step changes with the situation.
- UI texts are in English and short. Russian labels in the sketches `docs/ui/*.md` give the meaning, not the final text.

## Keyboard

- Pads: `1 2 3 4 / Q W E R / A S D F / Z X C V`, always, on all screens, except when a text input line is open (`/` opens it, Enter or Esc closes it).
- Commands: right hand, arrows, F1–F12 (screens), Space, Enter, Esc, Tab, Backspace, Ctrl+letter.
- Shift is only pad hit velocity.

# 03c — Choose the pad with arrows

## Why

On the files screen, after Enter takes a file in hand, the arrows still move through the file list. The owner expects them to choose the pad the file goes to (owner feedback, 2026-10-28).

## What changes

- While a file is in hand on the files screen, ↑ ↓ ← → move the selected pad on the grid next to the list; PgUp/PgDn change the bank.
- Enter puts the file on the selected pad (with the overwrite question if the pad holds something).
- Hitting a pad still puts the file there at once; Esc puts the file back, the list selection is where it was.

## Out of scope

Choosing pads with arrows on other screens: the pads screen already does it.

## How to verify

1. F6, select a file, Enter: the note says to choose a pad with the arrows.
2. → → ↓: the selection on the grid moves, the list does not.
3. Enter: the file goes to the selected pad (a question first if it holds a sample).
4. Take another file, Esc: the list selection is still on that file.

# 30 — Files: a held arrow scrolls, the preview talks

## Why

The owner (2026-10-31, a screenshot of F6): the folder path sits at the top left, above
the pads, while the list it names is on the right — "it is unclear why the folder is here
and the list is to the right". Holding the down arrow moves the selection one line and
stops. The preview off by default (23) reads as "the preview broke".

## What changes

- **A held arrow scrolls**: holding ↑ / ↓ (or I / K) keeps moving the selection, as the
  held arrow walks the cursor on the chop screen.
- **The preview is on by default**: the selected file sounds right away; `Space` turns
  it off and on, as before. The choice is not remembered across restarts.

## Out of scope

The folder path above its list: it moves to 31 (one layout for every screen). Remembering the preview toggle; the compact pads on the files screen (BACKLOG #15).

## How to verify

F6, hold ↓: the selection runs down the list; stop on a file — only that
file sounds. Restart, F6: the file under the selection sounds without `Space`; `Space` —
silence. Tests: recorded repeat events for the held
arrow, the preview default.

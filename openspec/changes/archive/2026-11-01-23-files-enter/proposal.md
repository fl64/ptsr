# 23 — Files: `Enter` loads straight to the pad, the preview rests

## Why

Loading a file is three presses (F6, `Enter` "in hand", the pad) and the browser stays
open until `Esc`. The grilling (2026-10-30) put loading files second in frequency, right
after chopping. The preview auto-plays by default: on a quiet setup every cursor move in
the list is sound the owner did not ask for.

## What changes

- **`Enter` loads to the selected pad**: on a file it loads it onto the selected pad and
  closes the files screen — two presses to a sounding pad. `Ctrl+Enter` keeps the "in
  hand" flow for when the pad is not decided yet. Folders still open on `Enter`.
- **The preview is off by default**: `Space` toggles it on and off, as before. Nothing
  sounds while walking the list until it is turned on.

## Out of scope

Drag and drop; the compact pads on the files screen (BACKLOG #15); rename pad
(BACKLOG #16).

## How to verify

Live: F6, arrows, `Enter` — the file plays on the selected pad and the screen is back on
pads; `Ctrl+Enter` — "in hand", a pad hit puts it there; walk the list — silence; `Space`
— the list talks, `Space` again — silence. Tests: recorded events for both file flows
and the folder case, and for the preview default, on and off; the hint text matches the
new `Enter` meaning.

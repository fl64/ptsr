# 41 — Enter takes the file in hand, Ctrl+Enter puts it on the selected pad

## Why

The owner (2026-10-31), checking 35: "I think picking a sample should be Enter and then a
pad hit, no?" — first the sound, then the pad, chosen after hearing it. The scheme of 23
is turned around: "Ctrl+Enter straight onto the pad".

## What changes

- `Enter` on a file takes it in hand; a pad hit puts it there; `Enter` again puts it on
  the selected pad; `Esc` puts it back.
- `Ctrl+Enter` on a file loads it onto the selected pad and closes the files screen.
- Folders open on either key.
- The first-steps hint and the files hint say the new way.

## How to verify

F6: `Enter` on a file — "in hand", hit W — the file lies on W; `Ctrl+Enter` on a file —
on the selected pad, back to F2. Tests: the key resolution, both flows, the hints.

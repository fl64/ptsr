# 22 — Notices: the startup warning clears, the save speaks

## Why

Two findings from the live poke (2026-10-30):

- The startup warning about the missing kitty protocol lives forever and sits on the
  notice line, so the next-step hints never show on terminals without the protocol
  (tmux, gnome-terminal…). The owner reads it once — after that it is noise.
- `Ctrl+S` saves without any feedback, and the autosave is silent too, so "did it
  save?" is never answered on the screen.

## What changes

- **The startup warning clears on the first action**: the first keypress of any kind
  removes it; a short marker "no releases" stays in the status line for the whole
  session. The hints come back as soon as the warning is gone. (Owner: clear on the
  first action, not on a timer.)
- **Manual save speaks**: `Ctrl+S` shows "Session saved" for 5 seconds. The autosave
  stays silent — it must not pop a message in the middle of playing. No dialog: a
  session save is not destructive; dialogs are for overwriting sample files
  (BACKLOG #19).

## Out of scope

A dialog on save; overwriting sample files (BACKLOG #19); other startup notices — they
keep the current behavior.

## How to verify

Live in tmux (no kitty protocol): the warning shows, one keypress clears it, the hint
line comes back, the status keeps "no releases"; press `Ctrl+S` — "Session saved" for
5 seconds. Tests: the notice dismisses on the first recorded key event, the marker is in
the status, the manual save flashes for 5 seconds, the autosave does not flash.

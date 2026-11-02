# 65 — The F12 Keys tab: rebind a key the way a controller is learned

## Why

The owner (2026-11-01): "the main thing is that it is handy". Editing a TOML file means
knowing action names and risking a typo; the controller is learned on F12 by picking an
action and moving a control. The keys should work the same way.

## What changes

- A Keys tab on F12 lists the actions by group with their keys, the owner's marked.
- Pick an action, `Enter`, press a key: it stands next to the old one in the pending
  colour; `Enter` binds, `Esc` keeps, as the learn mode does (57).
- A key taken in the same section: the action that loses it lights, the line warns.
- A pad key is refused with the reason in words.
- `Backspace` takes the key away; "reset to shipped" brings the shipped key back for one
  action or for all.
- Everything is saved in the user keymap of 64 at once.

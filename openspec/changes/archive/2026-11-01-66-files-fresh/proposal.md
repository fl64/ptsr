# 66 — The files screen: a fresh list, a visible filter, the name over the wave

## Why

A UX check of the owner's agent (2026-11-01), walked as a beatmaker; the owner: "yes" to
the proposal.

- A sample saved from a DAW while the program runs does not appear on F6: the folder is
  read when it is opened, not when the screen is entered, and there is no refresh key.
- A filter typed with `/` and closed with `Enter` stays on, and nothing says so: the list
  looks short for no reason the next day.
- The wave of the files screen names only the length (`0.33 s`), not which file sounds.

## What changes

- Entering F6 reads the open folder again; the selection stays on the same file if it is
  still there.
- While a filter is on, a line over the list says `filter: sn · Esc clears`.
- The wave of the files screen names its file next to the length.

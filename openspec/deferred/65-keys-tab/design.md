# Design 65

## Decisions (owner, 2026-11-01)

- The F12 tab on top of the user keymap (64), "handy" first.

## Defaults taken by the agent

- The tab reads like the MIDI tab: groups that fold, the group of the screen it came from
  open; the same `Enter` / `Esc` / `Backspace` and the pending colour of 57.
- An action may have several keys (letters and arrow twins): `Enter` adds the new key in
  place of the picked one; the row shows them all.
- While a key is awaited every key is taken as the answer, the pad keys too (they are
  refused with the reason); `Esc` alone cancels.

## Open questions

None yet.

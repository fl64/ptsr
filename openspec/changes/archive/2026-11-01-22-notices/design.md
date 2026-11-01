# Design 22

## Model

- The startup notices keep their order; the first key event of the session (any key, any
  screen) clears the whole startup list. One shared flag, no per-notice bookkeeping: the
  owner reads them together or not at all.
- The status line gains a fixed "no releases" marker shown while the terminal has no
  kitty protocol — independent of the notice line, so it survives the dismissal.
- The save flash is an ordinary flash message (5 s), like "MIDI connected".

## Decisions (with the owner, 2026-10-30)

- Clear on the first keypress, not on a timer: a timer can hide the warning before it is
  read; the eternal warning hides the hints forever.
- The marker stays in the status: the fact "no key releases, gate plays one-shot" matters
  for as long as the session runs.
- Flash, not a dialog: a dialog costs a keypress and steals the hands from the pads; a
  session save never destroys anything.

## Open questions

None.

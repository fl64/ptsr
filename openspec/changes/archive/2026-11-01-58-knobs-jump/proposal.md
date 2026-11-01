# 58 — Absolute knobs jump to their position; pickup only where a profile asks

## Why

The owner (2026-11-01): "when I start turning a knob it does not change the parameter at
once, `turn cc …` often flashes at the start". That is the soft takeover: the knob waits
until it reaches the current value, and the wait starts over on every new pad and every
page. Offered pickup (now), a catching-up scale and a jump, the owner took the jump.

## What changes

- A knob response setting on the MIDI tab of F12, one for all absolute knobs and faders:
  `jump` (the default: the parameter goes to the knob position at once), `pickup` (waits
  until the knob reaches the value, as now), `scale` (moves from the first touch, the knob
  and the value meet at the end stop). Kept in the user config.
- The device profiles no longer choose the response: their `takeover` is not read; the
  X-Session Pro profile drops it. Learned controls follow the same setting.
- `AGENTS.md`: the rule "absolute knobs and faders with soft takeover" becomes "the knob
  response from the settings, jump by default" (a separate commit).

## Trade-off the owner accepted

A knob that stands far from the value makes the sound jump at the first touch: a pad at
−12 dB with the volume knob at 0 dB gets loud at once. The glide removes the click, not
the jump.

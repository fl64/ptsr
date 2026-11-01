# 59 — The device profile seeds the bindings once; after that they are the owner's

## Why

The owner (2026-11-01): Backspace on the MIDI tab "does not remove the setting". The rows
he tried came from the X-Session Pro profile: Backspace removes only a learned binding, and
then the profile's role applies again, so a profile role can never be removed. The owner:
"the profile is a helper, it fills in the defaults, and then I want to change them as I
like".

## What changes

- The first time a device connects with no bindings of the owner's, the roles of its
  profile (its own or the default one) are copied into the user config. From then on the
  user config is the only source of roles for that device; the profile still names the
  controls (k1, b10) and their kinds.
- On the MIDI tab every row can be relearned or cleared with `Backspace`; a cleared row
  does nothing, no profile role comes back under it.
- A "reset to profile" action on the MIDI tab replaces the device's bindings with the
  profile's roles again.
- A config that already has learned bindings: the profile roles are added under them, a
  learned binding wins over a profile role on the same control or the same action.

## Not in this change

- The knob response (58), the sections like the Digitakt (open question).

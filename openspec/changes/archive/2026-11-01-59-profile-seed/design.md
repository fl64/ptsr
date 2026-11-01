# Design 59

## Decisions (owner, 2026-11-01)

- "The profile is a helper, it fills in the defaults, then I change them as I like" — and
  "ok" to: seed on first connect, every row clearable, reset to profile.
- The trade-off told to the owner: a profile changed by a later version of the program does
  not reach seeded bindings by itself; "reset to profile" brings it and drops the owner's
  changes.

## Defaults taken by the agent

- A seeded device is named in `[devices] seeded` of the user config. A config from before
  59 has bindings but no such list: its devices are seeded once on the next connect, the
  bindings winning; a device whose every row the owner cleared stays in the list, so it is
  not seeded again. (The first draft read "no table in `[midi]`": it cannot tell a device
  never seeded from one with bindings from before 59.)
- The seed happens when the device connects, in the UI thread, which owns the config; the
  MIDI thread is told the device is seeded and stops reading its profile roles.
- The `[edit]` layer of a profile is seeded into both edit groups, PADS · EDIT and
  CHOP · EDIT; the `[screens.*]` tables into their screen groups; the default profile's
  pads into the pads of the GLOBAL group.
- The F12 rows lose the "(learned)" mark: every binding is the owner's now.
- "reset to profile" asks first (Enter yes, Esc no): it drops every change of the owner's
  for that device.

## Archive order

"Device profiles" and the midi-learn requirements stand on the main specs; 59 is archived
after 58.

## Open questions

- Sections like the Digitakt: knobs k1–k3 and b3/b4 turn the sections of the selected pad
  with no edit open. The owner (2026-11-01): "leave it as it is for now" — the sections
  stay in the edit; kept in the backlog (row 21).

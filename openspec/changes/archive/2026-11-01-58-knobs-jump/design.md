# Design 58

## Decisions (owner, 2026-11-01)

- "V3": the jump, picked over the pickup (now) and the catching-up scale; the owner was
  told the jump breaks the takeover rule of `AGENTS.md` and makes the value jump at the
  first touch.

- Then (the owner's own proposal): the response as a setting with all three modes, `jump`
  by default (Q1.1); one setting for all knobs, no per-control choice in the profiles
  (Q2.1).

## Defaults taken by the agent

- `scale`: moving up from position p0 to p, the value v goes to
  v + (p − p0) · (1 − v) / (1 − p0); moving down, to v − (p0 − p) · v / p0. The first message
  of a control only notes its position. Once the knob and the value meet, the knob drives
  the value directly.
- The setting is a row on the MIDI tab of F12 and a key in the user config; no migration:
  absent reads as `jump`.
- The page knob (54) has no response: it is a switch.

- The scenarios "Pad change" and "Knob with soft takeover" keep their names (archive does
  not drop a scenario of a modified requirement); their text now says the knob jumps.

- "Zoom knob" (17) promised the view never jumps on a grab: that holds with `pickup`, with
  `jump` the view goes to the knob. A consequence of one setting for all knobs; the
  scenario says both.
- The setting lives under `[knobs] response` of the user config: `[midi]` holds the
  learned bindings as a map of devices and has no room for a plain value.

## Archive order

"Edit mode knobs in profiles" stands on 21, "Chop knobs" on 17: 58 is archived after 21.

## Open questions

None yet.

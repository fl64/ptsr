# 71 — No dead bindings

## Why

The owner (2026-11-01): "which buttons have no action?" Six bindings point at actions the
program does not have, and the program skips them without a word, so a key or a knob
looks taken and does nothing:

- `Ctrl+K` on every screen: `commands.palette`;
- X-Session Pro on the chop screen: k6 `slice.pitch`, k7 `slice.gain`, k8 `slice.reverse`,
  k9 `marker.nudge_coarse`, k10 `marker.nudge_fine`.

None of them is in an active change or in the backlog (the palette is only in
`docs/VISION.md`, which is not a plan). The X-Session ones were also copied into the
owner's config when the profile seeded it (59), so F12 shows those knobs as taken. The
owner: "remove the dead ones if they are not planned".

## What changes

- The six bindings leave `config/keymap.toml` and `config/devices/xsession-pro.toml`.
- A binding to an action the program does not have is named on the status line at start,
  for the keymap, the owner's keymap (64) and the device profiles, instead of being
  skipped silently.
- MIDI bindings in the owner's config that point at no action are dropped once at start,
  the config is saved without them, and the status says how many went.

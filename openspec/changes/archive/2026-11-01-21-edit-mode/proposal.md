# 21 — Edit mode and the parameter standard

## Why

Editing works, but the hands relearn it per screen: `−` `=` change a value on the pads
screen and zoom on the chop screen, `Tab` picks a parameter there and switches the auto
mode here, `M` marks pads there and cuts here. Knobs have no common story: outside MIDI
learn, `k1` turns the pad volume and nothing else is bound. The screen does not show
which layer is active or what a knob turns. The owner plays and edits at the same time —
the hands must not relearn per screen, and the eyes must see where they are.

Direction from the owner (2026-10-30): select a thing, enter an edit mode, the parameters
match to knobs and keys, the same keys mean the same everywhere, three knobs must be
enough; every mode is a layer with the same groups of keys, on the keyboard and on MIDI;
the active element is highlighted, also when a knob turns; convenient, fast, logical,
simple.

## What changes

- **Two hands**: the left fingers play, the left thumb holds `Ctrl` `Alt` for the right
  hand; the right hand moves with `J` `L` `I` `K` (the arrows too) and gives the commands.
  On the chop screen `J` `L` move the cursor, `I` `K` zoom, `Ctrl+J` `Ctrl+L` the left
  bound, `Alt+J` `Alt+L` the right bound; in an edit mode `I` `K` pick and `J` `L`
  change. A tap is the fine step, holding repeats the coarse one. Comfortable on the
  Ergohaven K:03 split and on a laptop (design.md).
- **Edit mode**: an explicit state entered on the selected object with `Enter` and left
  with `Esc` or a screen switch. Pads keep playing; editing never stops the music.
- **Parameter pages**: a page holds at most 3 parameters; knob k turns the k-th parameter
  of the current page — the same rule on every screen. The pads screen: TONE (volume,
  pitch) and ENV (attack, decay, release). The chop screen: BOUNDS (slice start, slice
  end). The page follows the picked parameter.
- **The screen shows the layer**: the edit mode colours the frame of what it edits and
  names the page; the picked parameter and the picked slice bound are highlighted; the
  knob numbers stand next to the parameters; a knob turn lights what it changed.
- **One letter, one meaning**: `M` cuts, `,` `.` set the region, `Y` the equal parts,
  `P` the assign, `N` the snap, `H` the pad mode, `M` the marking on the pads screen (the marker key: a cut on the chop screen); no command on the left
  half; `−` `=` retire; the arrows do not pick a pad.
- **The help in plain words**: groups named by the job, a key and a verb per item, the
  edit line shows only the edit keys; three bottom rows always kept for it.
- **The assign mode is deaf**: while the plan is open only the pads, `Tab`, `Enter`,
  `Esc`, the slice pick and the undo act; another key shows a hint how to leave.
- **The X-Session Pro profile** follows the layers: knobs 1–3 are the edit pages, buttons
  open the edit, go back and pick the parameter, the rest cut, walk markers, play and
  commit. (Moves here from 04b.)

## Out of scope

The settings screen keeps its own keys for now (a config list, not an edit mode); envelope
curve choices; the auto-chop threshold page (04d brings it); profiles for other hardware;
switching the keyboard pads off (they stay always on).

## How to verify

Live, hands on, on the split and on a laptop: play a beat with the left hand; `Enter` on
a pad — the panel is framed `EDIT · TONE`, the strip shows the piece with the envelope;
`I` `K` walk the parameters, `J` `L` change them (a tap fine, held faster), the knob
numbers stand on the page, a pad hit listens, `Enter` closes. On the chop screen `J` `L`
move the cursor, `I` `K` zoom, `PgUp` `PgDn` jump cut to cut; hold `Ctrl` / `Alt` with
the left thumb — the left / right bound turns amber — and move it with `J` `L`; `M`
cuts (on the playhead while it plays), `,` `.` set the region, `Ctrl ,` the whole, `Y`
equal parts; `O` steps the play mode and the green bar shows what `Space` plays; `P`
assigns, and during the assign `M` and the direction keys do nothing but hint. On the
pads screen `H` toggles gate (the release shows or dims), `M` marks pads, `J` `L` turn
the banks. The help line in three rows names a key and a verb.
Recorded-event tests cover the key roles, the pages, the highlight state, the steps and
the drawing; the feel is the owner's check.

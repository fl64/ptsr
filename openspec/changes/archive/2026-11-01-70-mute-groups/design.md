# Design 70

## Decisions (owner, 2026-11-01)

- "N2": the classic sampler way (MPC, SP-404): a group number per pad, a hit cuts the rest
  of the group.

## Defaults taken by the agent

- Eight groups, off by default. MPC has more; eight fit one knob turn and the circled
  digits `①`…`⑧`.
- The cut uses the engine's existing 5 ms fade, the one a pad uses to cut its own last
  hit; the pad's release time does not apply. A closed hat must cut at once.
- The group sits on the TONE page as its third parameter: the page has two, knob 3 is
  free, and the group is about how the pad plays, like the mode.
- A pad cuts only the others of its group; a pad hit again cuts itself as before.
- Sounds of the chop screen (listen, a slice, play as the pad) are not in groups: they are
  for listening, not for playing a kit.
- The scope takes the same group, not a relative step: a group is a name, as `H` gives
  the scope the same mode.
- The owner (2026-11-01): the group belongs under the other settings, not to the right
  of the mode; "windows differ in height: big today, smaller tomorrow, it has to adapt";
  "show that one can walk the settings up and down"; "the accordion is fine, but it may
  not fit either"; "the width may not fit either". So the group has its own row under the
  pitch, labelled `mute` (`mute off`, `mute 1`), and the panel fits the window: blocks
  fold (accordion), then scroll with `▲`/`▼`, and shorten their labels when narrow.
- The owner (2026-11-01): "Tab now walks the items, not the blocks", "Shift+Tab does not
  go back". With the edit closed `Tab` on the pads screen walked one parameter (21) and
  `Shift+Tab`, bound to nothing there, acted as `Tab`, forward. Now `Tab` and `Shift+Tab`
  turn the page whether the edit is open or closed. Then "up and down broke": with the
  edit closed only `Tab` had walked the parameters; `I` `K` and `↑` `↓` now walk them
  with the edit closed too, as in the edit.
- The owner (2026-11-01), on `change pad 1` under the folded FX block: "what does it
  belong to? it has to be explicit, maybe on top". The scope moves to the second row of
  the panel, under the pad and its bank, and says `scope` as the help line does; the row
  under the blocks is gone, so the panel is no taller.
- The owner (2026-11-01): "what threw me is that it was not the edit mode". The closed
  panel names the key that opens the edit (`Enter: edit`), the edit keeps its frame.
- The owner chose to keep the layout (2026-11-01, "Q2.1"): a taller window gives its rows
  to the strip, not to the panel; on TONE the FX block of a pad with effects stays folded,
  `Tab` opens it. Giving the panel the height is a change of the screen layout, not made.
- The empty pickup row is no longer drawn: it was the last row and was cut unseen in a
  40-row window anyway; the hint still appears while a knob waits.
- The scroll keeps no position of its own: the picked parameter decides what is seen, so
  `I` `K` move the panel and a page change brings its block in sight.
- The panel grows with the pad grid; a taller window gives its rows to the strip. A pad
  with effects therefore shows its FX block folded on TONE in any window (23 rows of
  room, 24 needed); the FX page unfolds it.
- In a 40-row window the TONE page now folds SOURCE and FX: "Every row in sight" (43)
  holds in a window tall enough for every row.
- A gate pad cut by its group stays silent until its next hit, its release does nothing.

- "For now the simple way, mute target maybe later": one group number, everyone in the
  group cuts everyone; a pad that cuts others without being cut by them (MPC "mute
  target") goes to the backlog (row 23). Added later as a `cuts` parameter that defaults
  to the pad's own group, it keeps the projects of 70 as they are.

## Open questions

None.

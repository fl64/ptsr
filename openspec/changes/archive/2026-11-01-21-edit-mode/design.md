# Design 21

## Model

Two hands, two roles (owner, 2026-10-30, round 5: "comfortable for two hands, on the
split and on a plain laptop"). A chord takes the modifier from one hand and the key from
the other, so no hand twists:

- the left fingers play the pads (the sound, the pick of a pad or a slice); the left
  thumb holds `Ctrl` and `Alt` for the right hand's keys and presses `Space`; the left
  little finger has `Esc` and `Tab`;
- the right fingers move and change: the direction letters `J` `L` `I` `K` (the arrows
  are the same keys for a laptop), the actions at the cursor `M` `,` `.`, the commands
  `Y` `P` `N` `H` `M`; `Enter` is the right thumb on the split, the right little finger
  on a laptop; the right hand holds `Ctrl` for the left hand's commands (`Ctrl+Z`).

| Keys | Chop screen | Pads screen | Edit mode (both) |
| --- | --- | --- | --- |
| a pad hit | plays its slice, selects it | plays the pad, selects it | plays, selects |
| `J` `L` (`←` `→`) | the cursor | the bank | the value |
| `I` `K` (`↑` `↓`) | zoom in / out around the cursor | — | the parameter |
| `Ctrl+J` `Ctrl+L` | the left bound of the selected slice | — | — |
| `Alt+J` `Alt+L` | the right bound of the selected slice | — | — |
| `M` | cut at the cursor (a marker) | marking on / off | — |
| `,` `.` | the region starts / ends at the cursor | — | — |
| `Ctrl+,` | the region is the whole sample | — | — |
| `N` | the cursor steps evenly / from hit to hit | — | — |
| `Y` | equal parts | — | — |
| `P` | the assign mode | — | — |
| `PgUp` `PgDn` | the cursor to the previous / next cut, or the start / end | the bank | — |
| `O` | the play mode: to the end / to the cut / the slice | — | — |
| `H` | — | the pad mode, one-shot / gate | — |
| `Tab` | in the assign: in order / by hand | the parameter | the parameter |
| `Enter` | the slice edit on / off; in the assign: apply | the pad edit on / off | closes |
| `Esc` | one layer back | one layer back | closes |

- A tap is the fine step, holding repeats the coarse step: the cursor, the bounds, the
  zoom and the values. No modifier for "fine": Ghostty takes `Ctrl+Shift+arrows` (tabs)
  and `Ctrl+Alt+arrows` (splits), and `Shift` with the right `Alt` switches the keyboard
  layout.
- The direction letters act like the arrows, so they differ per screen as the arrows do;
  every other command letter means the same on every screen it is bound on. No command
  lives on the left half (`1`–`5`, `Q`–`T`, `A`–`G`, `Z`–`B`): the left hand plays.
- No pad is picked with the arrows: a pad hit picks it. In the hand modes (a file or a
  pad in hand) a pad hit places and swaps; the arrows there do nothing.
- The edit mode keeps the right hand alone on the values while the left hand plays: no
  modifier is needed inside it. The bounds outside it are a two-hand chord (the left
  thumb holds the modifier), the quick way when nothing plays.
- The edit mode is a screen-local state: the pads screen edits the selected pad, the chop
  screen edits the selected slice. Switching the screen leaves the mode. The picked
  parameter survives `Esc` and the screen switch.
- A page is a group of at most 3 parameters of the edited object. Knob k turns the k-th
  parameter of the page — in every edit mode, on every screen. The page follows the
  picked parameter.
- A knob is a page slot, not a parameter binding: one learned knob follows whatever sits
  in its slot. Outside the edit mode the knobs keep their roles from the profiles and
  MIDI learn (`pad.volume`, the slice bounds on the chop screen).
- The left half stays pads-only in every layer; `Shift` on a pad is velocity. The
  keyboard pads are always on: editing never stops the music.
- `Ctrl+J`, `Ctrl+I`, `Ctrl+M` look like `Enter` and `Tab` without the kitty protocol;
  the program needs the protocol for the pad releases anyway.

## The help line in plain words

The owner asked for a help line that is easy to read (2026-10-30). It says what the
hands can do now, not the whole keymap:

- a group is a job the owner does (MOVE, CUT, BOUNDS, PADS, EDIT…), at most four groups
  on a screen, the most used first;
- an item is a key cap and a verb: "J L cursor", "M cut", not an action name;
- the direction letters are shown, the arrows are named once in F1;
- in the edit mode the line shows only the edit keys and the way out, not the screen's;
- a held modifier shows what it does with `J` `L` ("Ctrl: J L the left bound");
- a state is named once, where the screen does not show it ("snap: hits").
- the way back stays in sight (owner, 2026-10-30: "I pressed , . — how do I undo?"): the
  chop line shows `Ctrl ,` whole and `Ctrl Z` undo as hold hints, without "+", so the
  usual line keeps them.

## The screen shows the layer

Without a visible state the layers are guesswork (owner, 2026-10-30). The screen always
shows three things:

- **The layer.** The pad edit draws a magenta frame around the pad panel titled
  `EDIT · ENV` (the place of the frame is kept when it is closed, so nothing jumps); the
  slice edit puts `EDIT · BOUNDS` at the start of the slice line.
- **The object and the parameter.** The selected pad and slice are highlighted as now;
  the picked parameter is highlighted in the panel; on the chop screen the bound in play
  (a held `Ctrl` / `Alt`, the picked one in the edit, the one just moved) takes the amber
  bound colour, all lines thin. The cursor is a white line with `!`, the playhead a
  green thin line, what play covers a green bar above the wave.
- **The knobs.** In the edit mode every parameter of the page carries its knob number in
  the panel: `1 attack · 2 decay · 3 release`. The help line names the page and the keys.
- **What just changed.** A knob turn or a value key lights the changed parameter (and on
  the chop screen the moved bound) for about a second, in and out of the edit mode, so a
  knob outside the mode shows what it turned.

## Decisions (with the owner, 2026-10-30)

- Pages over "knob 3 follows Tab": a knob keeps its meaning while the page is up, and the
  ENV page fills exactly three knobs. Poking may overturn this — then the spec changes
  first.
- 04b pauses: its X-Session chop profile moves here; kitty graphics stays in 04b.
- The chop edit mode opens from the listen mode only; in the assign mode `Enter` stays
  commit.
- Grilling: `Enter` confirmed as the entry key; `Ctrl+E` stays the fallback only if the
  collisions feel wrong live.
- Grilling: pads keep playing while the edit mode is open — a requirement, not an
  accident: editing must not stop the music.
- Grilling: the assign mode is deaf — while the plan is open only the pads, `Tab`,
  `Enter`, `Esc`, the slice pick and the undo act.
- Spec gap found in the implementation: `Enter` was the only keyboard way to start the
  assign. The assign starts from the keyboard with `P` ("put on pads"); the `chop.commit`
  action keeps its dual meaning for device buttons (no plan — start, plan — apply).
- Third round, "convenient, fast, logical, simple" (owner: "do it the best way"):
  (Superseded in part by rounds 4–5 below: the quick path, the letters `K` `L` and the
  slice walk on `↑` `↓` are gone; the hint, the always-on pads and the layers stay.)
  - One layer table for all screens (above). Simpler beats the spare paths: `−` `=` and
    `,` `.` retire, `Alt+←` `Alt+→` is the one quick value path on both screens. This
    replaces "the mode never gates a keyboard path" and "`Alt+←` is the slice start,
    `Alt+→` the slice end": `Alt` changes the picked value, the picked bound is named on
    the screen.
  - One letter, one meaning: the pad mode leaves `G` (left half, next to pad `F`) for
    `H`; the marking leaves `M` (it cut on the other screen) for `K`; the equal parts
    leave `Tab` (it picked the parameter on the other screen) for `L`.
  - `↑` `↓` on the chop screen walk the slices (the object layer); they were bound to an
    action the program does not know and did nothing.
  - The screen shows the layer, the picked parameter, the knob numbers and what a knob
    just changed (section above). This replaces "the knob labels live in the help line
    only; no row under the panel".
  - The keyboard pads stay always on; the crowding of the right half is solved by the
    layers and the highlight, not by switching the left half to commands.
  - The assign mode answers an ignored key with a hint in the status line: "assigning:
    Enter applies, Esc leaves". Silence looked broken.
  - Device buttons map onto the layers like the keys: the edit entry, back and the
    parameter pick, so the whole edit loop runs from the controller.
- The edit layer lives in the keymap as its own `[edit]` table over the screen keys:
  the same keys on every screen, visible in one place of the config. (Its round-3 quick
  path `Alt+←` `Alt+→` is gone: `Alt` names the right bound since round 5.)
- The fine step of a pad (implementation, 2026-10-30): the envelope steps 1 ms instead of
  5 ms; volume and pitch keep 1 dB and 1 semitone — the session stores them whole, a finer
  step would change the session format.
- The help band (owner, 2026-10-30: "reserve the three bottom rows for the hotkeys, so
  nothing jumps"): the help line keeps 3 rows at the bottom on every screen; the screen
  above never moves when the help changes. The cost: the body loses a row or two where
  the help took one.
- No pad selection with the arrows (owner, 2026-10-30: "picking a pad with the arrows
  makes no sense, there are keys for that"): a pad hit selects the pad, the arrows of
  the pads screen are free. The hand modes lose the arrow choice too (pad-assignment
  delta): a pad hit places and swaps. The direction keys of the pads screen turn the
  banks (round 5).
- Key layout round 4–5 (owner, 2026-10-30): the chop arrows as the owner wants them
  (`Ctrl` the left bound, `Alt` the right bound, up and down zoom), on letters for both
  hands (the model above); "slower with `Shift`" became "a tap is fine" because of the
  Ghostty and layout chords. The split is not the reference, the laptop must be as good
  (owner). This replaces the round-3 "`Alt+←` `Alt+→` change the picked value outside the
  mode" and "`↑` `↓` walk the slices".
- The cursor is the owner's (owner, 2026-10-30: "the cursor does not run away while it
  plays, I move it, not the sound"; "play always plays from the cursor"): playback starts
  from the cursor every time and draws its own playhead; stopping leaves the cursor; a
  cut while playing lands on the playhead and leaves the cursor. Before this, a cut while
  playing landed on the cursor, the start of the playback (found in the code).
- The bound in play has a colour, not a weight (owner, 2026-10-30: "when I hold the
  modifier, light the bound it moves; just another colour, not a bold line"): all slice
  lines are thin; the bound the keys move now (a held `Ctrl` the left, `Alt` the right,
  else the picked one in the edit, else the one that just moved) takes the bound colour.
  A held modifier is seen with the kitty protocol only.
- Play modes (owner, 2026-10-30: "to the end, to the nearest cut, the slice under the
  cursor; show above or below with a clear line what plays now; a key switches"): `O`
  steps them (free on every screen, right ring finger over `L`); a green bar right above
  the wave shows the span; it opens in "to the end"; not saved (the session format stays).
- A held key speeds a bound up (owner, 2026-10-30: "a tap moves a little, held it speeds
  up, but not too fast"): first a tap ~4 ms growing to ~12 ms per repeat — replaced the
  same day by "One speed" below (shares of the view). Ghostty sends a held key's repeats as plain presses (recorded in
  `ghostty_auto_repeat_does_not_retrigger`), so with the kitty protocol a second press
  without a release counts as a repeat; without the protocol there are no releases and
  every press stays a tap. Found here: until now "held is coarse" never fired in Ghostty.
- One speed for the cursor and the bounds (owner, 2026-10-30: "the bounds and the cursor
  move about the same at any zoom; the bounds speed up too slowly and end too slow"; "pick
  the speed"): steps are shares of the visible wave — a tap 1/128 (about a cell), a held
  key from 1/128 to 1/32 per repeat over 6 repeats. At the system's 25 repeats a second
  (`xset q`: delay 660 ms, rate 25) the top speed crosses the screen in about 1.3 s. The
  660 ms before the first repeat is the system's, not the program's. This replaces the
  4 ms → 12 ms bound ramp.
- Jump to a cut (owner, 2026-10-30: "PgUp PgDn put the cursor on the next / previous bound
  if there is one, or on the start / end"): `PgUp` back, `PgDn` forward, as the keys
  usually go; the page of slices follows the cursor instead of turning on `PgUp` `PgDn`;
  in the assign they stay the banks. On the split `PgUp` `PgDn` sit on layer 1.
- The release by the mode (owner, 2026-10-30, screenshot: "is the release right? there is
  no sample there"; "make it clear; and if I switch gate off, the behaviour differs?"):
  a one-shot pad ignores the release, so it is not drawn and the panel dims it with
  "(gate)"; a gate pad shows it apart from the wave as a "key up" fall. Before, a dotted
  tail after the piece started from full level next to a decay at zero.
- `M` is the marker key (owner, 2026-10-30: "mark the pads with M, not U"): it marks the
  pads on the pads screen and marks a cut (a marker) on the chop screen — the one letter
  with two jobs, kept on purpose: both are "set a mark". A habit from the chop screen only
  turns the marking on, which the help line shows and `M` turns off. `U` is free.
- The edit mode and the assign mode never stand together: a device commit with the edit
  mode open does nothing, the same as `P`; `Esc` closes the help (F1) before the edit mode.

## Live poke findings (owner, 2026-10-30)

- "Which bound does Backspace delete?" — the one a held modifier lights: `Ctrl+Backspace`
  the start, `Alt+Backspace` the end (owner's words: "Ctrl / Alt light the bound, so
  Ctrl / Alt + Backspace delete it"). Plain `Backspace` keeps deleting the start. A
  cursor on a cut deletes that cut: pinned by a scenario.
- Deleting all cuts with one key: offered, dropped by this answer; `Ctrl+Z` and equal
  parts stay the ways to start over.

## Decisions after the audit (owner, 2026-10-31)

- "By the contract": the slice under the cursor plays once when the cursor enters it (the
  arrows, the jump by cuts), as "The cursor selects" says; the code only selected it.
- The X-Session Pro gets the cursor and the region on knobs 3–5 (they held names of
  actions the program does not have). Its commit button starts the assign in order from
  the selected pad, the second press applies: a controller has no pads to tap, and a
  sampler lays slices onto pads in a row by convention. `P` stays the assign by hand.

## Open questions

- The letters (`H` mode, `M` marker, `Y` parts, `P` assign, `N` snap, `O` play mode) and
  the speeds (a tap 1/128 of the view, held up to 1/32; `RAMP_REPEATS` in the chop) are
  the owner's to poke live (task 18); what feels wrong is changed in the spec first.
- `PgUp` back, `PgDn` forward was picked by the usual direction; the owner's words named
  them "next / previous" in that order — to confirm live.
- X-Session Pro (owner, 2026-10-30, "go" on the table): an `[edit]` layer in the profile
  over the screen roles while the mode is open, not buttons free on every screen (the chop
  screen has none). b10 toggles the edit (`pad.edit` / `chop.edit` outside, `ui.back`
  inside), b3 b4 pick the parameter as `↑` `↓` do, k1–k3 are the page slots. In the pad
  edit b3 no longer steps the pad mode. No separate back button. The physical places of
  b3, b4, b10 are not known yet; the owner checks them on F12.
- Envelope times on a knob: a square curve over 0–5000 ms; linear would give 39 ms per
  knob step at the short end. To poke live (task 18).
- The `[edit]` layer stands over learned controls too: a learned knob would otherwise
  keep `pad.volume` inside the edit and break the slot rule.
- Chop page: {slice start, slice end}; the third knob stays empty until 04d brings the
  threshold.
- The settings screen keeps its own keys for now; its edit mode is a later talk.
- A moving dot of the sounding voice on the pad envelope: needs the voice position from
  the engine; a later talk.
- An envelope of a slice on the chop screen (an ENV page in the slice edit; the profiles
  already name `slice.attack`, `slice.fade`, `slice.fade_curve`): offered to the owner,
  not asked for yet; a change of its own.
- The program once broke the system sound (2026-10-29, 21:33): after a start the
  Bluetooth headphones got no cycles, the browsers fell silent; a reconnect did not help,
  the restart of the user audio services and of the players did. Not reproduced twice
  with music playing (graph snapshots: the USB microphone drives the graph while the
  program runs, no errors). Untried: a start right after a quit (the 21:33 log shows a
  quit at :32 and a start at :33). Not part of 21. If it happens again: keep the program
  open and capture the graph live before any repair.
  Second time, 2026-10-30 09:01 MSK (snapshot in `target/audio-incident/`, not in git):
  the program quit at 09:01:01 and started at 09:01:14; WirePlumber logged "scheduling
  stopped node" at 09:01:14, the same second. After that every node, the headphones
  included, had quantum 0 and rate 0 (no cycles), though the headphones were connected and
  the browser links were active. The restart of the user audio services fixed it: silence
  played on the headphones at 2048/48000, the program started after it runs in the graph.
  Earlier quit-then-start pairs that day (05:53:26 → 05:53:38, 05:54:00 → 05:54:01 UTC)
  broke nothing as far as the owner saw. The program log is in UTC, the system log in MSK.
  Live check the same morning (10:08, the program running, sound fine): the USB microphone
  (driver priority 2009) drives the graph at 1024/48000, the headphones (priority 1010)
  and the program follow it; links: microphone → program inputs, program outputs →
  headphones. The program links the microphone at every start, recording or not
  (`audio.rs`, `route`). Suspect: the driver handover between the Bluetooth and the USB
  clocks at start and quit; not proven — no control run yet.
- Bluetooth delay (2026-10-30, ~14:40–15:00 MSK): on HUAWEI FreeBuds 7i (AAC) a pad hit
  was heard "about a second later"; Xiaomi Buds 5 Pro (SBC) felt fine; after the owner
  reconnected the Huawei it was fast again. PipeWire's own latency figures did not explain
  it (Huawei 200 ms, Xiaomi 380 ms reported). The program path was the same in both
  cases. Suspect: delay building up in the Bluetooth link, reset by the reconnect; not
  proven. First step next time: reconnect the headphones.
- A key held while the terminal loses focus may lose its release: the next press of that
  key then counts as a repeat once.
- The pads screen needs 26 rows of the terminal since the help band; at 24 the last row
  of the grid is cut.
- Archive order: 04b before 21 — the 21 delta modifies "Pad envelope", which 04b adds.

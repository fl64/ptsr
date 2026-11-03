# Plan: SMC-PAD integration and the command panel (changes 91–94)

Read `README.md`, `KEYMAP-CLEANUP.md` and `REFACTOR-INPUT.md` first. This track needs
keymap cleanup 83a and changes 84–90 done: one route
for pads, `Focus` / `Mode`, `ui.confirm`, tap / hold buttons, relative encoders.

## 1. What the SMC-PAD gives the computer

Reference maps from other people's measurements (Windows USB, Bluetooth), carried from
`openspec/deferred/83-controller-profiles/design.md`. These reports were not independently
verified during this import. **Change 91 checks each fact on the owner's Linux machine first.**

Sources to retain when 91 replaces 83:

- Manufacturer manual: https://manualf.oss-cn-hongkong.aliyuncs.com/manual/MIDI-KEYBOARD/SMC-PAD_ZE.pdf
- Reported MIDI measurements: https://github.com/SDR3078/mvave-home-assistant/blob/main/docs/HARDWARE-BLE.md
- Independent preset map: https://github.com/zhrepot/SMC-PAD-for-VirtualDJ/blob/main/README.md

| Control | Sends | Note |
|---|---|---|
| 16 pads, layer 1 | notes 36–51, channel 10, velocity, real note-off | note 36 is the bottom-left pad |
| 16 pads, layer 2 (PAD BANK) | notes 52–67 | PAD BANK itself sends nothing |
| 8 encoders, bank 1 | CC 30–37, channel 1 | factory: absolute 0–127 counter kept in the device, it stops at the ends |
| 8 encoders, bank 2 (KNOB BANK) | CC 38–45 | KNOB BANK itself sends nothing |
| LEFT, RIGHT, PLAY, STOP, RECORD | CC 25–29, channel 1, 127 on press, 0 on release | |
| SHIFT, PAD BANK, KNOB BANK, Note Repeat, BT | nothing | local to the device |
| aftertouch | channel pressure after hits, amount is a device setting | ignore it |
| pad lights | only when the pad's Led field equals a note; velocity is the color index | on USB, only seen on the third port with MCP pads; Linux not tested |
| ports | Windows shows three port pairs; "SMC-PAD" is a part of all three names | Linux names unknown |

Consequences:

- The device SHIFT is useless to ptsr. ptsr makes its own modifier: hold STOP (change 92).
- PAD BANK and KNOB BANK are useful: ptsr knows the layer from the note or CC number.
- The encoders must be in relative mode. The owner sets the type "CW" (relative) for the 8
  encoders once in MidiSuite. MidiSuite runs on Windows or macOS. If the owner cannot do this,
  stop and ask: in absolute mode the knobs stop at the ends and the plan does not work.
- A port match by the text "SMC-PAD" can match several ports. Change 91 decides the match.

## 2. The layout

Principles (they go into the spec of change 92):

1. Outside the command panel, layer 1 pads sound or act as targets / steps where the mode
   says so, as keyboard pads do. Only an explicit panel hold changes them into command cells.
2. Commands are on the 5 buttons, on the command panel, and on the knobs.
3. One role per button on every screen.
4. A frequent action is 1 press; any action is not more than 2 presses. A pad hit that picks
   the object or the target (the pad to edit, the pad to load onto) is not counted.
5. A second press of the same panel cell confirms its own step (as `P` then `P` today). A
   question is answered by `ui.confirm`.
6. The screen always shows what the panel cells and the knobs do now.

### 2.1 Buttons

| Button | Tap | Hold |
|---|---|---|
| PLAY | `play.toggle`: play / stop what the screen is about: the transport (Pads, Pattern), the selection (Chop), the preview (Files) | from change 95: `metronome.toggle` |
| STOP | `ui.back` (Esc) | `layer.panel`: the command panel while held |
| RECORD | `record.toggle`: the microphone recording; from change 97, the pattern recording when the transport plays or the Pattern screen is open | `record.mic` while held (as `Alt+R` after 83a) |
| LEFT / RIGHT | `nav.prev` / `nav.next`: the main list of the screen: bank (Pads), page (edit), slice (Chop), folder parent / open (Files), bar page (Pattern) | repeat |

While STOP is held, the other buttons change meaning (from change 93a): LEFT is undo, RIGHT is
redo, PLAY is panic (all sound stops). RECORD with STOP held does nothing.

`play.toggle`, `record.toggle`, `nav.prev`, `nav.next` are new context actions. Each maps to an
existing action per screen and mode; the table lives in one function in `crates/input`, with a
test per row. On Pads before the transport exists (95), PLAY plays the selected pad.

### 2.2 The command panel

Hold STOP: the 16 layer 1 pads become 16 cells. PAD BANK layer 2 shows the same cells as a
latch. The pad grid on the screen is replaced by the panel grid, with the labels in the physical
layout (top row of pads = top row of cells). Release STOP: pads sound again at once.

```
 ┌──────────┬──────────┬──────────┬──────────┐
 │  Pads    │  Files   │  Chop    │ Pattern  │  fixed: screens
 ├──────────┼──────────┼──────────┼──────────┤
 │  c5      │  c6      │  c7      │  c8      │  context of the screen
 ├──────────┼──────────┼──────────┼──────────┤
 │  c9      │  c10     │  c11     │  c12     │  context of the screen
 ├──────────┼──────────┼──────────┼──────────┤
 │  FX      │ Settings │  Undo    │ Projects │  fixed
 └──────────┴──────────┴──────────┴──────────┘
```

The Pattern cell is empty (dim) until change 98. Context cells:

| Screen / mode | c5 | c6 | c7 | c8 | c9 | c10 | c11 | c12 |
|---|---|---|---|---|---|---|---|---|
| Pads | Edit | Mode | Swap | Clear | Mark | Mark bank | Hide pads | Rec output |
| Pads · Edit | Page − | Page + | Param − | Param + | Reset pad | — | — | Close edit |
| Files | Take | Load to pad | Open | Parent | Favorite | Preview loop | Chop file | Rec output |
| Chop | Cut | Parts | Hits | Assign | Snap | Play mode | Loop | Delete cut |
| Chop · Assign | In order / by hand | Bank − | Bank + | Apply | — | — | — | Cancel |
| Chop · slice edit | Start ← | Start → | End ← | End → | Delete end | Normalize | As pad | Close |
| FX | FX 1 | FX 2 | Kind | Bypass | — | — | — | — |
| Settings | Tab | Learn | Clear binding | — | — | — | — | — |
| Any screen · question | Yes (`ui.confirm`) | — | — | — | — | — | — | No (`ui.back`) |

Cells name logical actions, not raw MIDI controls. Most exist today (see `Action` and the
keymap); others, such as FX bypass and the context actions, still need a separate behavior
spec and tests before they can be bound. Do not add placeholder bindings. The
table is data: `config/panel.toml`, read like the keymap, so the owner can change it. A cell may
name a layer action or a confirmable action. The panel definition is validated at start: an
unknown action is refused with a message, as for the keymap.

Panel safety still needs the owner's decision in change 92. Layer 2 is latched: its cells
need no held STOP, so a panel cell alone is not a sufficient guard. Proposed default: Clear,
Delete cut, Reset pad and replacement Apply open a visible question in either panel form;
only the question's Yes cell calls `ui.confirm`. Keep the X-Session safe direct defaults
and their tests unchanged. Specify the SMC panel exception separately; do not remove the
X-Session rule. See this folder's README, "Review notes before implementation".

### 2.3 Knobs

Bank 1 (CC 30–37) is the context of the screen. Bank 2 (CC 38–45) is the same on every screen.

| Knob | Pads / Pads · Edit / FX | Files | Chop | Pattern (from 98) | Settings |
|---|---|---|---|---|---|
| 1 | page parameter 1 | scroll the list | cursor | tempo | rows |
| 2 | page parameter 2 | — | slice start | swing (later) | values |
| 3 | page parameter 3 | — | slice end | quantize | — |
| 4 | page | — | zoom | quantize strength | — |
| 5 | — | — | hit threshold | pattern length | — |
| 6 | — | — | region start | — | — |
| 7 | — | — | region end | — | — |
| 8 | master volume | master volume | master volume | master volume | master volume |

Bank 2, every screen: selected pad volume, pitch, cutoff, decay, send FX 1, send FX 2, tempo
(from 95; before it, empty), master volume.

Hold STOP while turning a knob: the fine step (as Shift on the keyboard), no acceleration.

### 2.4 Paths for the main stories (the target of change 93)

Here `P` names one command gesture, not one physical press: STOP+cell is two presses.
Change 93 counts physical presses per command, including picker and question paths.
These story sketches are not yet proof that the reach rule passes.

| Story | Command gestures (P) and turns (T) |
|---|---|
| Load a file onto a pad | STOP+Files (P), knob 1 (T), PLAY to hear (P), STOP+Take (P), hit the pad (P) |
| Chop by hits and lay out | STOP+Chop (P), knob 5 (T), STOP+Assign (P), STOP+Assign again to apply (P) |
| Set a pad's decay | hit the pad (P), bank 2 knob 4 (T) |
| Swap two pads | hit pad A to select it (P), STOP+Swap (P), hit pad B (P) |
| Open a project | STOP+Projects (P), knob 1 (T), `ui.confirm`: second press of STOP+Projects (P) |
| Change the bank | LEFT / RIGHT on Pads (P) |
| Record the microphone onto a pad | hold RECORD (P), hit the pad (P) |

Change 93 makes this table for every story of `docs/USER-STORIES.md` and fixes what is longer
than the rule.

## 3. The changes

### Change 91 — `91-smc-pad-profile`

Goal: the SMC-PAD works as a set of pads, knobs and buttons with the current actions. It
replaces the deferred change 83.

Step 0, with the owner (before any code; write the results into `design.md`):

1. The owner sets the 8 encoders (both knob banks) to relative ("CW") in MidiSuite and keeps
   the rest of the factory preset. Write down the preset number.
2. The owner connects the SMC-PAD by USB to Linux and runs `aconnect -l` and
   `aseqdump -p <port>`, or opens F12 → input debug. Record: the port names; the bytes of each
   pad on both PAD BANK layers; each encoder on both KNOB BANK banks, slow and fast, both ways;
   each of the 5 buttons, press and release; what aftertouch arrives.
3. If the bytes differ from §1, the measured bytes win. Update §1 in this file.

Tasks:

1. `config/devices/smc-pad.toml`: a complete profile. Pads: layer 1 notes 36–51 as `p1..p16`
   with the physical order of design 83 (notes 48–51 are screen pads 1–4, top row; 36–39 are
   13–16). Layer 2 notes 52–67 as buttons `c1..c16` (the panel uses them in 92; in 91 they are
   unassigned). Encoders `e1..e8` CC 30–37 and `f1..f8` CC 38–45, `type = "relative"` with the
   measured encoding. Buttons `left right play stop record` CC 25–29.
2. Port match: match the one port that carries notes and CCs. If Linux shows several ports,
   add an optional `match_index` or match a more exact name; spec delta in `input-devices`.
3. The profile has a `channel` filter for the whole device, but the pads are on channel 10 and
   the rest on channel 1. Keep the filter off for this profile (design 83 decided this). Test
   both channels.
4. Ignore channel pressure (aftertouch) in the parser path for this profile; test that it
   triggers nothing.
5. Default roles with today's actions (no panel yet): STOP tap `ui.back`, PLAY tap the chop or
   preview play, LEFT / RIGHT bank or slice, RECORD hold `record.mic`, knobs as in §2.3 where the
   action exists.
6. Raw-byte tests through the shipped TOML, profile seeding, dispatch and `App` (the harness of
   84): pads with velocity, both note-off forms, relative steps, buttons 127 / 0, two devices
   (X-Session + SMC-PAD) at the same time, unplug during a held pad.
7. Delete `openspec/deferred/83-controller-profiles`; carry its sources into this design.
8. The virtual controller tool `task midi-replay` and the first scripts in `tests/midi/`
   (`CROSS-CUTTING.md` §1.1).

How to verify (owner): hit every pad on both layers; turn every knob on both banks; press every
button; check F12 input debug and the default roles.

### Change 92 — `92-command-panel`

Goal: §2.1 and §2.2: the buttons with tap / hold and the command panel.

Spec deltas: a new capability `command-panel`, plus `input-devices` (button roles, the
"Chopping from the X-Session Pro" rule edit), `screen-layout` (the panel grid in place of the pad
grid while the panel is active), `screen-help` (the bottom line shows the 5 button roles of the
connected controller).

Tasks:

1. `Mode::Panel` in `Focus` (added by the MIDI thread at the hold moment, and by the UI from the
   layer event). `route_pad(Panel)` returns `Ui`: no sound.
2. `config/panel.toml` and its parser and validation; one table per screen and mode, the fixed
   rows once.
3. A panel cell hit → the cell's action through the normal `App::apply`.
4. Layer 2 notes `c1..c16` → the same cells, latched (no hold). The screen shows the label
   `PANEL` in the title line while the last note came from layer 2; a hit on layer 1 clears it.
5. Context actions `play.toggle`, `record.toggle`, `nav.prev`, `nav.next`: one table function,
   a test per row.
6. Drawing: the panel grid in `crates/tui`, the same size as the pad grid, cell labels from the
   action short names; empty cells dim; the cell of the last action lit for `LIT_FOR`.
7. Scenarios (parity harness where a keyboard way exists): hold STOP shows the panel; release
   hides it; hold STOP + Files cell opens Files; tap STOP is back; a pad hit with STOP held does
   not sound; Assign twice from the panel lays out the slices; layer 2 latch works and the pads
   of layer 1 sound again after a layer 1 hit.
8. Visual (`VISUAL.md`): the `PANEL` badge; the panel grid with destructive cells in magenta
   text and latched cells in the pad-mode color; the `L2` / `K2` layer marks in the status line;
   the action echo of controller presses; hints by the device of the last input; the `SMC` help
   group only while the SMC-PAD is connected. Tui tests for each.
9. Two controllers and unplugging (`CROSS-CUTTING.md` §2.2, §2.3): the layer belongs to its
   device; unplug ends it; the X-Session tests of change 82 stay green.

How to verify (owner): hold STOP and look at the screen; go through every screen by the top row
of cells; chop and lay out a break without the keyboard.

### Change 93 — `93-controller-reach`

Goal: prove the goal of the owner and close the gaps.

Tasks:

1. `docs/CONTROLLER-PATHS.md`: for every story in `docs/USER-STORIES.md`, the controller path
   and the count of presses, as in §2.4.
2. A "text-only" list in `crates/input` (an `ActionInfo` flag): `pad.rename`, `browser.filter`,
   the typed name of a new project, `app.quit`, keymap edits. Nothing else may be on it without
   the owner's approval.
3. The reach test, in both directions. Controller side: for each screen and each mode, for each
   action that the keyboard can do there and that is not text-only, find a controller path in the SMC-PAD profile + `panel.toml`:
   direct button (1 press), panel cell (2), context action (1), knob (turns). Fail with the list
   of actions without a path or with more than 2 presses. Keyboard side: every action that a
   controller can do has a key or a key path too, except the controller-only list (hold STOP for
   the panel, the layer marks). The test also checks that every mode has its badge
   (`VISUAL.md` §2.1).
4. Fix the gaps the test shows. Expected ones: the project picker by knob and confirm; the
   folder picker of the Paths tab; the settings values by knob; the splash closes on any
   controller input; help (F1) as a panel cell or another explicit path. Do not reserve PLAY
   hold for help: change 95 uses it for the metronome. Decide the help path in the design.
5. The rule for later changes goes into `AGENTS.md`: a new action names its controller path in
   its spec, and the reach test covers it.

How to verify (owner): work a full session from start to a saved project with the keyboard out
of reach: load, chop, lay out, shape, effects, save, switch projects.

### Change 93a — `93a-controller-comfort`

Panic, redo, a new project with a date name, the pad velocity curve, the controller settings in
F12, the first-connect notice, the F1 SMC-PAD page and `docs/SMC-PAD.md`. The full list is in
`CROSS-CUTTING.md` §2.1 and §3.

### Change 94 — `94-pad-lights` (optional experiment)

Goal: the SMC-PAD pads show state with color. Do this only if a short experiment works.

Step 0: on Linux, send note-on to each SMC-PAD output port with the Led field set (MidiSuite)
and see if a pad lights. If nothing lights in one hour of tries, stop: write the result into the
design and close the change as "not possible now".

If it works:

1. A MIDI output thread with a queue from the UI (no MIDI output from the audio thread).
2. A light model: per pad a color from the state: empty, loaded, playing, selected, marked,
   panel cell, step on / off (later), the playhead step (later).
3. Send only changes, not 30 times a second; resend all after a reconnect.
4. Lights are a help, not a need: everything works without them.

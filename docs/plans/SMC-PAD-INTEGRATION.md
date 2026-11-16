# Plan: SMC-PAD integration and the command panel (changes 91–94)

Read `README.md`, `KEYMAP-CLEANUP.md` and `REFACTOR-INPUT.md` first. This track needs
keymap cleanup 83a and changes 84–90 done: one route
for pads, `Focus` / `Mode`, `ui.confirm`, tap / hold buttons, relative encoders.

## 1. What the SMC-PAD gives the computer

The change 91 design lists reference maps. Change 91 read the active preset on
the owner's Linux machine and captured a small MIDI sample. The physical position of every
control still needs physical checks. A one-record USB write and exact restore passed.
The application also wrote all 16 encoder records. A USB script restored the original preset.

Sources retained in the change 91 design:

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
| ports | Linux: `SINCO SMC-PAD-Private`, `SINCO SMC-PAD-Master`, `SINCO` | Private and Master mirrored the captured note and CC. Use Master for performance. |

Consequences:

- Reports say SHIFT sends no MIDI message. Its physical Linux check is still open.
  Only a measured SHIFT message can enable the optional quick panel view.
- PAD BANK and KNOB BANK are useful: ptsr knows the layer from the note or CC number.
- The active preset has 16 absolute encoder records. Change 91 tries a guarded USB setup:
  save the records, change only those records to relative mode, and read them back. A failed
  check leaves encoder actions off and reports the failure. The write and restore path
  passed on this unit. Physical encoder output still needs a check.
- A match by the text `SMC-PAD` can match both inputs. Give Master the performance profile
  and Private an empty profile so mirrored messages do not double actions.

## 2. The layout

### One navigation pattern

This is the target flow for control-pages, changes 88–93, and later screens. It
does not describe a complete SMC-PAD profile in the current build.

1. Hit a pad to select the thing to work on: a pad, slice, step, channel, or slot.
2. Press PAD BANK to select pad layer 2. The first layer 2 hit shows the command
   panel if it is hidden. It runs no command. Hit a cell after the panel appears.
   The main page selects screens and shows four current-screen commands. More
   opens a context page. The bottom row keeps Back, Undo, Redo and Panic.
3. Turn knob 1 in bank 1 to move the selection or cursor. Knobs 2–8 change
   parameters of the selected thing. The knob strip names their current roles.
   Bank 2 controls the selected pad and the master on every screen.
4. Hit the Back panel cell to close an edit or return from a screen to Pads.
   Press PAD BANK to return to pad layer 1. The next layer 1 hit hides the panel
   and uses the normal pad route.
5. Press PLAY to play or continue what the screen shows. Press RECORD to record
   the pattern. Press LEFT or RIGHT to move through the screen's main list.
   Tap STOP to pause the transport or stop a preview. Tap it again while
   paused to return to the start. PLAY continues after the first STOP.

The keyboard keeps its full path: F keys, Enter, Esc, I/K, J/L, and Tab.
Panel access never changes the transport or stops a sounding voice. During play,
knob bank 1 controls the screen and bank 2 controls the selected pad and master.
Knobs keep working while the panel is visible.

The first panel entry takes PAD BANK and two layer 2 hits. Later cell commands
take one hit while the panel is visible. Change 93 counts these physical presses
and records the first-entry cost. A frequent action remains direct where a
button, pad, or knob has that role. Do not count the first reveal as a command.

Performance tools such as roll, pad mute, slot pick, note repeat, and erase use
latched pad modes. One panel cell enters a mode. The same cell or Back leaves it.
The badge names the mode. Destructive cells always ask a visible question.
Entering or leaving a mode does not change the transport.

### 2.1 Buttons

| Button | Tap | Hold |
|---|---|---|
| PLAY | Play or continue the transport on Pads and Pattern, the selection on Chop, or the preview on Files. | Toggle the metronome. |
| STOP | Pause the transport or stop a preview. A second tap while paused returns to the start. | No default role. Change 93a can add an optional panel hold. |
| RECORD | Record the pattern. | Record the microphone while held. |
| LEFT / RIGHT | Previous / next item in the screen's main list. | Repeat. |

STOP has no hold binding by default, so its tap acts on press. Change 89 adds
tap and hold for PLAY and RECORD. It does not bind a default layer to STOP.
PLAY never stops an active transport. Back and Undo are active fixed cells.
Redo and Panic keep fixed places but remain unavailable until change 93a.
Gesture slots use command actions. Physical pad triggers keep plain bindings
and their direct release path. Change 89 does not assign SMC-PAD buttons until
change 91 measures their Linux MIDI messages.

ButtonRoles in control-pages uses this table. Each row gets a keyboard path and
raw MIDI tests through the real profile and dispatch. The context action table
maps PLAY, STOP, RECORD, LEFT, and RIGHT on each screen and mode.

### 2.2 The command panel

PAD BANK latches physical pad layer 2, notes 52–67. PAD BANK itself sends no
MIDI message. The first layer 2 hit while the panel is hidden only shows it.
Later layer 2 hits run the named cells. A layer 1 hit hides the panel and follows
the normal pad route. The screen cannot react to PAD BANK alone. It learns the
new layer from the next pad note. The status line shows PANEL while the last
pad note came from layer 2.

The panel replaces the pad grid with 16 cells in the same physical layout.
The main page gives cells 1–8 to Pads, Files, Chop, Pattern, FX, Settings,
Help and More. Cells 9–12 hold four current-screen actions. More opens a
context page with up to ten other actions, Next and Main cells. Next cycles
through further context pages. The bottom row
keeps Back, Undo, Redo and Panic in cells 13–16. Change 92 checks access to
Files, Chop, Pattern, FX and Settings. Redo and Panic remain dim until 93a.
No cell acts on the first layer 2 hit when the panel is hidden.

A setup guide and ptsr doctor tell the owner to give layer 2 pads a distinct
color in MIDI Suite. The color is a device setup aid, not a requirement for
panel access on Linux. The on-screen PANEL mark remains the source of truth.

When a command needs ordinary pad targets, press PAD BANK to return to layer 1.
Target hits hide the panel and use the active screen mode. Press PAD BANK again
to return to layer 2. The first layer 2 hit reveals the hidden panel without
action; the next hit can run Apply or another cell. The reach check records
these switches in the complete story path.

SHIFT has no host shortcut in 92. Its printed pad shortcuts can change the
device preset or octave. Only a distinct measured MIDI message could support
a later quick look. Change 93a can add an optional STOP hold path. PAD BANK
remains the only panel entry path in 92.

#### Panel state cases for change 92

| Situation | Required result | Check |
|---|---|---|
| Transport plays and first layer 2 note arrives | Show panel. Run no cell. Keep sound and transport. | Raw MIDI sequence and owner check |
| Another layer 2 note arrives while panel shows | Run exactly its cell once. | Raw MIDI sequence |
| Layer 1 note arrives after panel use | Hide panel. Route this hit as a normal pad hit. | Raw MIDI sequence |
| Assign needs target pads | Change to layer 1 for targets. Change back and reveal the panel before Apply. | Raw MIDI sequence and owner check |
| A second device sends a pad while SMC panel shows | Keep that device's pad route and sound. | Two-device sequence |
| SMC-PAD disconnects while the panel shows | Clear PANEL and release its held gates. | Disconnect sequence |
| Redo or Panic cell is pressed | Show unavailable. Run no action until 93a. | Raw MIDI sequence |
| Destructive cell asks. | Keep playback. Only P8 or Yes runs it. | Raw MIDI sequence. |
| A question is visible and P7 or P8 is pressed | Cancel or confirm on the first hit, even if the panel was hidden. | Raw MIDI sequence |
| STOP has no hold | STOP tap pauses or stops a preview on press. It never shows the panel. | Raw MIDI sequence |

| Screen / mode | Example middle-row cells |
|---|---|
| Pads | Edit, Mode, Swap, Clear, Mark, Mark bank, Hide pads, Record output |
| Pads edit | Page previous, Page next, Parameter previous, Parameter next, Reset pad, Close edit |
| Files | Take, Load to pad, Open, Parent, Favorite, Preview loop, Chop file |
| Chop | Cut, Parts, Hits, Assign, Snap, Play mode, Loop, Delete cut |
| Chop assign | In order / by hand, Bank previous, Bank next, Apply, Cancel |
| Chop slice edit | Move start, Move end, Delete end, Normalize, As pad, Close |
| FX | FX 1, FX 2, Kind, Bypass |
| Settings | Tab, Learn, Clear binding |
| Question | Yes, No |

Cells name logical actions, not raw MIDI controls. The panel map is data in
config/panel.toml. It is validated at start. An unknown action is refused with
a message. A destructive cell asks before it removes or replaces anything.
Only the question's Yes cell confirms it. Keep the X-Session direct safe
defaults and tests unchanged.

### 2.3 Knobs

Bank 1 (CC 30–37) is the context of the screen. Bank 2 (CC 38–45) is the
same on every screen.

| Bank 1 knob | Pads | Pads edit | Files | Chop | Pattern | FX | Settings |
|---|---|---|---|---|---|---|---|
| 1 | selected pad | selected parameter | selected file | cursor | selected row or step | selected slot | selected row |
| 2–8 | pad context | page parameters | file context | zoom, threshold, slice bounds | pattern parameters | slot parameters | selected setting |

The knob strip names each available parameter. It dims a knob when that screen
or mode has no parameter for it. The selected thing and knob 1's role remain
visible.

Bank 2 has the same roles on every screen. Knob 1 controls pad volume, 2 pan,
3 pitch, and 4 cutoff. Knob 5 controls decay, 6 send FX 1, 7 send FX 2, and
8 master volume. Features that do not exist yet stay dim until their own
changes add them. A controller fine-step path needs a decision in change 90.
The default panel does not use STOP as a knob modifier.

### 2.4 Paths for the main stories (the target of change 93)

Each panel-cell action below assumes that the panel is visible. On the first
visit, press PAD BANK and hit any layer 2 pad once to reveal it. This first hit
does not run a cell. Change 93 measures the complete first-use path and the
shorter repeated path separately.

| Story | Controller path |
|---|---|
| Load a file onto a pad | Files cell, knob 1 to select, PLAY to hear, Take cell, hit the target pad |
| Chop by hits and lay out | Chop cell, knob for threshold, Hits cell, Assign cell, PAD BANK to layer 1, target pads, PAD BANK to layer 2, reveal hit, Apply cell |
| Set a pad's decay | Hit the pad, turn bank 2 knob 5 |
| Swap two pads | Hit pad A, Swap cell, hit pad B |
| Open a project | Projects screen cell, knob 1, confirm cell |
| Change the bank | LEFT or RIGHT on Pads |
| Record the microphone onto a pad | Hold RECORD, hit the pad |

The planned control-pages change shows a page for both "Chop the sound of a
pad" and "Edit a pad". Its unit test checks that bank 1 knob 1 moves the
selection or cursor. Change 93 includes both stories as named paths in its
reach test.

## 3. The changes

### Change 91 — `91-smc-pad-profile`

Goal: the SMC-PAD works as a set of pads, knobs and buttons with the current actions. It
replaces the deferred change 83.

Step 0: record the results in `design.md` before code.

1. Read the active preset over USB and save the original encoder records before any write.
   Test one reversible record, then set all 16 relative records only if read-back matches it.
   Keep the rest of the preset and every other slot unchanged.
2. Read Linux port names and capture MIDI from all three inputs with `aconnect -l` and
   `aseqdump -p <port>`, or F12 input debug. Record the port names.
   Record pad bytes on both PAD BANK layers. Record slow and fast encoder bytes
   on both KNOB BANK banks in both directions. Record press and release bytes
   for each of the five buttons. Check whether SHIFT changes any message.
   Record aftertouch.
3. If the bytes differ from section 1, the measured bytes win. Update section 1 here.

Tasks:

1. `config/devices/smc-pad.toml`: a complete profile. Pads: layer 1 notes 36–51 as `p1..p16`
   with the physical order of design 83 (notes 48–51 are screen pads 1–4, top row; 36–39 are
   13–16). Layer 2 notes 52–67 as buttons `c1..c16` (the panel uses them in 92; in 91 they are
   unassigned). Encoders `e1..e8` CC 30–37 and `f1..f8` CC 38–45, `type = "relative"` with the
   measured encoding. Buttons `left right play stop record` CC 25–29.
2. Port match: use Master for performance. Give the mirrored Private input an empty profile.
   Do not match the generic third port.
3. The profile has a `channel` filter for the whole device, but the pads are on channel 10 and
   the rest on channel 1. Keep the filter off for this profile (design 83 decided this). Test
   both channels.
4. Ignore channel pressure (aftertouch) in the parser path for this profile; test that it
   triggers nothing.
5. Roles before the panel exists: LEFT / RIGHT use existing selection actions where
   available. RECORD hold uses `record.mic`. PLAY and STOP stay unassigned until
   change 92 adds their start-only and stop-only context actions. Existing toggle
   actions cannot meet the agreed PLAY and STOP behavior. The keyboard keeps Esc
   for back until change 92.
6. Raw-byte tests through the shipped TOML, profile seeding, dispatch and `App` (the harness of
   84): pads with velocity, both note-off forms, relative steps, buttons 127 / 0, two devices
   (X-Session + SMC-PAD) at the same time, unplug during a held pad.
7. The change 91 design retains the source links, pad orientation and raw-route tests.
   Change 91 removed the superseded deferred draft during the archive.
8. The virtual controller tool `task midi-replay` and the first scripts in `tests/midi/`
   (`CROSS-CUTTING.md` §1.1).

How to verify (owner): hit every pad on both layers; turn every knob on both banks; press every
button; check F12 input debug and the default roles.

### Change 92 — 92-command-panel

Goal: implement the button roles and the latched pad-layer panel of section 2.
STOP has no panel hold in 92. Change 93a can add that optional path.

Spec deltas: command-panel, input-devices, keyboard, transport, screen-layout
and screen-help. State the button roles, the first-hit rule, the panel cells,
the status mark and the P7/P8 question paths before code. Keep keyboard and
X-Session paths.

Tasks:

1. Route layer 2 notes to panel state. The first
   layer 2 hit when the panel is hidden only reveals it. A later hit runs its
   cell through the normal action path. A layer 1 hit hides it and uses the
   normal pad route. PAD BANK itself has no event.
2. Read and validate config/panel.toml. Define the fixed Back and Undo cells.
   Reserve dim Redo and Panic cells for 93a. Define main and context pages.
   Show the cell labels in the physical 4×4 order.
3. Give PLAY, STOP, RECORD, LEFT and RIGHT their screen roles. STOP has no
   default hold binding. Its first tap pauses transport. A second tap while
   paused returns to the start. Space keeps its stop-and-reset path.
4. Add the PANEL status mark while the last pad note came from layer 2.
   Keep the knob strip active. Draw the panel in the pad-grid space.
5. Keep SHIFT local and leave the optional STOP hold to 93a. A visible
   question takes P7/P8 before the first-hit reveal rule. PLAY and STOP
   never answer a question.
6. Make existing performance tools available through the panel where their
   actions exist. A destructive cell always asks a visible question.
7. Test raw bytes through the shipped profile and dispatch. Enter and
   leave the panel while the transport plays. Check that sound and transport
   do not change. Check that the first layer 2 hit runs no cell, later hits
   do, and STOP pauses or stops without starting. Check that a layer 1 hit
   leaves the panel and follows its normal route.
8. Test two controllers and disconnects. The panel belongs to its device.
   A disconnect clears a held shortcut. The X-Session tests stay green.
9. Add catalogue views at both sizes for the panel, layer mark, question,
   and existing performance modes.

How to verify (owner): start playback. Press PAD BANK and hit a layer 2 pad
once. The panel appears and no command runs. Hit a screen cell and use its
commands. Turn both knob banks while music plays. Hit Back, then a layer 1
pad. Check that the music never stops on panel entry or exit. Tap STOP:
playback pauses at its position. Tap it again: the position returns to the
start. Check Undo in a disposable project. Redo and Panic remain dim until 93a.

### Change 93 — `93-controller-reach`

Goal: prove the goal of the owner and close the gaps.

Tasks:

1. `docs/CONTROLLER-PATHS.md`: for every story in `docs/USER-STORIES.md`, the controller path
   and the count of presses, as in §2.4.
2. A "text-only" list in `crates/input` (an `ActionInfo` flag): `pad.rename`, `browser.filter`,
   the typed name of a new project, `app.quit`, keymap edits. Nothing else may be on it without
   the owner's approval.
3. The reach test checks both directions. For each screen and mode, find a
   controller path for each non-text keyboard action. Use the SMC-PAD profile
   and `panel.toml`:
   direct button, pad hit, panel cell, or knob. Count a visible panel cell as one
   press. Count the first panel entry separately: PAD BANK and two layer 2 hits.
   Report any action without a path or any repeated path above two presses.
   Keyboard side: every controller action has a key or key path, except layer
   selection, its status mark, and an optional measured SHIFT quick look.
   The test also checks that every mode has its badge
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

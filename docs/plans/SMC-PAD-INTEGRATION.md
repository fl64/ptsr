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

This section records the layout after the review of PR #27.
Change 124 adds the new button and knob roles and the knob strip.
See [the route table](../CONTROLLER-PATHS.md) for the current panel paths.
Change 125 adds the main menu, target routes and device warnings.

### 2.1 Buttons and selection

Hit a pad to select a pad, pattern row or slice. Turn knob 1 to move within the selection.
Use LEFT and RIGHT for the main list of the current screen or mode.
The knob strip and one-row help name the list that the buttons move.

| Screen or mode | LEFT and RIGHT | Bank 1 knob 1 |
|---|---|---|
| Pads | Pad banks | Neighboring pad without sound |
| Pad edit | TONE, ENV, FILTER, SOURCE and FX pages | Parameter on the page |
| Pattern | Previous and next bar page | Step in the selected row |
| Chop and Chop edit | Previous and next slice | Cursor |
| Files | Parent folder and enter the selected folder | File or folder in the list |
| Settings | Tabs | Row in the tab |
| FX | Effect slots | Parameter in the slot |

On Files, RIGHT acts only on a folder. Turn knob 1 to select a file. Use the existing
Take or Load action to use it. Pickers keep their own selection and confirmation routes.
Knob 1 moves projects only while the project picker is open.

| Button | Tap | Hold |
|---|---|---|
| PLAY | Start or continue transport. On Files and Chop, start the screen preview. | Toggle the metronome. |
| STOP | Pause transport or stop a preview. A second tap while paused returns to the start. | No action. |
| RECORD | Record a pattern where available. | Record the microphone. |
| LEFT / RIGHT | Move in the current list. | Repeat the same move. |

Use the keyboard and MIDI learn paths for the existing actions. P7 and P8 answer
questions. PLAY and STOP keep their playback roles during a question. STOP acts on
press and has no panel role.

### 2.2 The command panel

Change 124 replaces the panel steps for values, parameters and edit pages with
direct controls. Change 125 uses the main menu and target routes below.

Enable `SMC-PAD PAD BANK` by default on the supported USB unit after its status-read
check passes. F12 MIDI offers `Open panel` and `Off`. A detected press opens or
closes the panel before the next pad hit. The device retains its selected layer
after release. If the panel stays hidden, its first command-layer hit only shows it.
A playing-layer hit closes the panel. The status line shows PANEL while it is open.

Read the main rows from top to bottom and the cells from left to right.

1. Pads · Files · Chop · A1.
2. FX · Mixer · Setup · More.
3. Up to four actions for the current screen.
4. Back · Undo · Redo · Panic.

A1 opens Pattern. Setup opens Settings. Mixer stays dim until `mixer-view`.
Open Controller Help from More with P10. F1 still opens Help. More opens a context page. The Next and
Main cells move between panel pages. LEFT and RIGHT keep their screen roles.
Panic uses only the double press from 93a. Its cell stays in the bottom-right corner.

Keep related actions side by side in a physical row. Put the earlier, lower or
negative action on the left. For questions, P7 is No and P8 is Yes.
Keep Undo to the left of Redo. Apply the same order to minus and plus, and to Previous
and Next when both are on a panel page. Back closes the current mode first.
Use the normal question route for an action that removes or replaces a target.

Use the glossary in [UI style](../ui/STYLE.md) and a list of allowed cell labels.
Each label fits within six terminal cells. Use `knobs` for the A1 controls and
`record` for pattern recording. Use `open` for the project picker, with
`Open project` in the help text. `projects` exceeds the label limit.
Use `grid -` and `grid +` for grid size. Keep product names such as A1 and FX.
Remove the panel steps for values, parameters and edit pages when a direct route replaces them.
Keep their keyboard and MIDI learn actions. Knobs change values. LEFT and RIGHT change edit pages.

Use P1–P12 on the target panel for Take, Copy, Swap, file placement and Assign.
Use the playing layer for P13–P16. P13 cancels on the target panel.
P14 applies an Assign plan. P15 switches between targets and commands.
A playing-layer hit closes the panel and retains the current target route.

The [route table](../CONTROLLER-PATHS.md#target-routes) compares the software
press counts before and after the change. Record actual counts on the device
during the combined owner check. Include any P7 or P8 question answer.

### 2.3 Knobs

KNOB BANK sends no event on its own. The last knob CC shows which bank sent it.
The owner reports that the encoders have no detents. Use buttons for page,
bank and tab moves. The right hand plays the pads. The left hand can reach the
bottom row first and find knob 8 in the top-right corner.

#### Bank 1 (CC 30–37)

Knob 1 moves position. Knobs 2–7 change up to six values on the current page.
Knob 8 always controls master volume. Change 126 removes acceleration: each detent
is one step. Check that a slow zoom turn gives a small change.

Change 126: on Pads, knobs 2–7 have no role outside pad edit. In pad edit, knob 1
selects the page. Knobs 7, 5 and 3 change the page values in screen order, from top
to bottom. Give Pattern, FX and Chop their own value pages.
Remove the global roles for project scroll, tempo and bank selection from knobs 1,
7 and 8. Use knob 1 for project scroll only in the picker.

Put tempo on the Pattern transport page with swing, quantize and length.
Pattern More uses P6 `tap` for Tap Tempo. The keyboard uses Alt+B.
Files, Chop and Settings have no tempo knob.
While ptsr Note Repeat is on, bank 1 knobs 1–4 use RATE, SWING, TEMPO and LATCH.
These roles follow the printed labels. Keep this override tied to change 100.
Change 126 leaves bank 2 with only master.

#### Pairs and page examples

The rows, from bottom to top, are 1–2, 3–4, 5–6 and 7–8.
Use the bottom row for position and zoom or the main value. Use rows 3–4 and 5–6
for two pairs. Put a single value beside master on the top row. Keep the start or
lower value on the left. Give each pair a name and an order in the page table or
panel data. Test that the members are neighbors on the same row and in order.
Also test the allowed cell labels and six-cell width.

| Page | Knobs 1–2 | Knobs 3–4 | Knobs 5–6 | Knobs 7–8 |
|---|---|---|---|---|
| Chop | Cursor · zoom | Slice start · slice end | Region start · region end | Threshold · master |
| Pad edit TONE | Page · — | Mute group · — | Pitch · — | Volume · master |
| Pad edit ENV | Page · — | Release · — | Decay · — | Attack · master |
| Pad edit FILTER | Page · — | Resonance · — | Cutoff · — | Filter · master |
| Pad edit SOURCE | Page · — | End · — | Start · — | Source · master |

Change 126 replaced the pad page pairs. A page lists its values in one column, so
the left knob column follows that column from top to bottom.

#### Bank 2 (CC 38–45)

Change 126 removed these pad roles. Only knob 8 (master) stays. The table
records the layout of change 124.

| Physical row | Left knob | Right knob |
|---|---|---|
| Top | 7 Pan | 8 Master |
| Second | 5 Send FX 1 | 6 Send FX 2 |
| Third | 3 Pitch | 4 Decay |
| Bottom | 1 Volume | 2 Cutoff |

Bind pan when `pad-sound` adds it. Dim pan until then. Use the same rule for a send
or page value that has no available action. Show the proposed changes before an existing saved profile uses the new roles.
Keep explicit owner bindings and removals. Ask about saved roles with unknown origin.

#### The knob strip

Show both banks side by side. Draw each bank as the physical grid in this order:
7–8, 5–6, 3–4, 1–2. Highlight the bank of the last turned knob. Label it as the
last input. Name the current page and the list that LEFT and RIGHT move. Check the
strip and one-row help at 80 by 26 and 120 by 40.

### 2.4 Routes, preset checks and owner decisions

The [route table](../CONTROLLER-PATHS.md) keeps the shipped paths and lists the new goals.
Aim for two presses for a screen change, three for a context command and zero for a
value turn. Count the fallback reveal and each additional context page separately.
Check the complete target flows on the physical SMC-PAD during owner review.

Use the numbers printed on the controller when explaining SHIFT. Printed PAD1–8
select presets. PAD9–12 set velocity curves. PAD13–16 set transpose and octave.
The swing labels refer to the separate device Note Repeat mode.
See the [manufacturer manual](https://fccid.io/m/db5542b15b2598e8c9dd9db080d605408b44fc685456f1f797e95e1da99ab5b8.pdf).
The panel's P7 and P8 use the screen grid order.

- An unassigned note outside the supported pad ranges on channel 10 gives one
  warning per Master connection. Use `SMC-PAD pads are shifted (SHIFT +
  PAD13–16 changes octave or transpose). Reset it on the device.` The warning
  cannot identify a shifted note that still matches the profile. Do not infer a
  different pad map.
- Read the active preset slot with the existing USB check. If it differs from the
  supported slot, show the preset number in a warning. Disable knob actions as in
  the encoder record check.
- Explain the device Note Repeat clock in the setup guide and the F1 SMC-PAD page.
  Use ptsr Note Repeat from change 100 for the supported repeat path.

[Change 124](../../openspec/changes/124-smc-knob-pages/) covers the buttons, both
knob banks and their labels. To update a saved old map, open F12 MIDI and select
`SMC-PAD layout review`. Compare the current and proposed roles. Accept one change
or all listed changes. Use Esc or P7 to cancel.
[Change 125](../../openspec/changes/125-smc-panel-polish/) adds target selection and device warnings.

| Before | Owner decision | Proposed default |
|---|---|---|
| `smc-knob-pages` | LEFT and RIGHT move the main list on each screen or mode. | Yes. On Files use parent and enter folder. |
| `smc-knob-pages` | Bank 1 uses K1 for position and K2–K7 for current values. | Yes. |
| `smc-knob-pages` | K8 always changes master in both banks. | Yes. |
| `smc-knob-pages` | Put tempo on the Pattern transport page and Tap Tempo on a context cell. | Yes. Keep the printed Note Repeat roles. |
| `smc-knob-pages` | Use RATE, SWING, TEMPO and LATCH on K1–K4 in ptsr Note Repeat. | Yes, in bank 1 when change 100 is available. |
| `smc-knob-pages` | Place each pair on one row with the lower or earlier value on the left. | Yes. Test each pair. |
| `smc-knob-pages` | Use the bank 2 order in §2.3 for the selected pad. | Yes. Dim unavailable actions. |
| `smc-knob-pages` | Apply new roles to an existing profile after review. | Keep conflicts for the owner to resolve. |
| `smc-panel-polish` | Enable PAD BANK detection on supported USB. | Yes, after its status-read check passes. |
| `smc-panel-polish` | Use direct targets on the command layer for P1–P12. | Test the full layout and count presses during review. |
| `smc-panel-polish` | Move Help from the main page to More. | Yes. |

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

Goal: implement the first latched pad-layer panel and the context buttons.
See the archive of change 92 for the initial roles. STOP has no panel hold in 92.

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
5. Keep SHIFT local. A visible question takes P7/P8 before the first-hit reveal
   rule. PLAY and STOP
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
2. Mark `pad.rename` and `browser.filter` as text entry in `ActionInfo`. A typed new-project
   name and keymap edits are text operations outside that list. Keep `app.quit` outside
   `ACTIONS` and MIDI learn. Give the SMC-PAD an explicit Exit context cell with a visible
   question. P8 confirms it. P7 or Back cancels it. Ctrl+Q keeps its direct exit behavior.
   No other exception may enter the reach inventory without the owner's approval.
3. The reach test checks both directions. For each screen and mode, find a
   controller path for each non-text keyboard action. Use the SMC-PAD profile
   and `panel.toml`:
   direct button, pad hit, panel cell, or knob. Count a visible panel cell as one
   press. Count first panel entry separately, with and without the extra reveal hit.
   Report any action without a path or any repeated path above two presses.
   Keyboard side: every sampler action has a key or key path. PAD BANK selection,
   its status mark, panel page navigation and an optional measured SHIFT quick look
   are controller interface steps, not sampler actions.
   Bind Alt+U to transport toggle on every screen. Keep Space preview on Files and Chop.
   On Pads and Pads edit, bind Alt+Left and Alt+Right to silent pad selection.
   Keep the existing Alt-arrow slice-bound keys on Chop.
   On Chop and Chop edit, bind Ctrl+Alt+Up and Ctrl+Alt+Down to exact 1 dB
   threshold steps. Compare them with encoder 3, including bounds and Undo.
   The test also checks that every mode has its badge
   (`VISUAL.md` §2.1).
4. Close the gaps that the production-route test shows. Expected gaps include project
   selection by encoder 1 and P8, Paths folder selection, and Settings values by encoder 2.
   Help needs navigation after the existing panel cell opens it. Keep the accepted rule
   that sound ends the splash. A silent controller action does not close it. PLAY hold
   keeps its metronome role. Test Exit while a recording or save is pending.
5. `AGENTS.md` already requires a controller path in the spec for each new action. Add the
   reach test that enforces the rule for later changes.

Owner steps without the keyboard:

1. Load a sound and chop it.
2. Assign slices to pads, shape the sound and add effects.
3. Save the project and switch projects.
4. Select Exit and press P8.

Keyboard check: on Files and Chop, use Alt+U for transport and Space for preview.
Repeat Alt+U in the Russian layout. Check the same transport result from page 2
`trnspt` on the SMC-PAD. On Pads and Pads edit, compare Alt+Left and Alt+Right
with the SMC-PAD LEFT and RIGHT buttons. A pad selection must not play sound.
On Chop and Chop edit, compare Ctrl+Alt+Up and Ctrl+Alt+Down with encoder 3.
Check the threshold, cuts, linked pads, -40 and 0 dB bounds, and Undo.

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

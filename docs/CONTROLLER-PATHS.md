# SMC-PAD paths for current stories

This table covers the headings in [User stories](USER-STORIES.md).
It describes the SMC-PAD profile and command panel for changes 124 and 125.
Raw MIDI tests check these paths. Physical press counts remain open.

## Count a command

The counts give **visible main panel / hidden panel with reveal**. The hidden
count includes one PAD BANK press and one hit to reveal the panel. On verified
USB, F12 MIDI defaults to `Open panel`. A detected press opens or closes the panel.
The next detected press returns to the playing grid without a pad hit.

| Action | Main panel visible | PAD BANK opens the panel | PAD BANK and a reveal hit |
| --- | --- | --- | --- |
| Main cell | 1 | 2 | 3 |
| Context page 1 cell | 2 | 3 | 4 |
| Context page 2 cell | 3 | 4 | 5 |
| Context page 3 cell | 4 | 5 | 6 |
| Context page 4 cell | 5 | 6 | 7 |
| Knob turn in the active bank | 0 | 0 | 0 |
| LEFT, RIGHT, PLAY, STOP or RECORD | 1 | 1 | 1 |

More opens context page 1. Next advances one page. Count KNOB BANK separately
if a turn needs the other knob bank. Count target pad hits separately.
A visible modal question takes P7 or P8 on the command layer. Add one PAD BANK
press if the device is on the playing layer. No reveal hit is necessary.

| User story | SMC-PAD route and command counts | Exception or limit |
| --- | --- | --- |
| Terms | Read the screen and pad labels (`0/0`). | Observation. |
| Two hands | Playing pads trigger sound. LEFT and RIGHT select banks (`1/1`). Knob 8 controls master in both banks (`0/0`). | Pad hits are separate. |
| SMC-PAD command panel | Enter with PAD BANK. Use a main cell (`1/3`) or More and a context cell (`2/4`). Next advances the context page. | The USB shortcut can omit the reveal hit. |
| Chop the sound of a pad | Select a pad. Open Chop (`1/3`). Knob 1 moves the cursor (`0/0`). Main Cut, Parts, Hits, or Assign (`1/3`). Context page 1 Commit (`2/4`). PLAY previews (`1/1`). | Count target pads and answers separately. |
| Edit a pad | Select a pad. Main Edit (`1/3`). LEFT or RIGHT changes the edit page (`1/1`). Turn knobs 7, 5, 3 and 1 for the page values, from top to bottom (`0/0`). | Back closes the edit. |
| Read modes and operation results | Main Copy or Back (`1/3`). More, then P10 Help (`2/4`). In Help, P7/P8 select tabs and P9/P10 move (`1/1`). | Badges, results, and current knob roles are observations. |
| Read pad playback | Hit a playing pad. Read PLAY (`0/0`). | Pad hits and releases are separate. |
| Your own keys | Edit `~/.config/ptsr/keymap.toml`. | Keyboard-only document edit. |
| The splash screen | Open a screen from a main cell (`1/3`). Play a pad in the playing layer. | Splash text is an observation. A silent hit keeps it open. |
| Play a beat | Hit playing pads. PLAY and STOP control transport (`1/1`). Pattern More, P6 Tap sets tempo (`2/4` for the first tap). | A Gate release belongs to its original pad. |
| Name a pad | Press `Ctrl+N` and type a name. | Keyboard-only text entry. |
| Copy a pad without a growing name | Main Copy (`1/3`). Select a target on the panel. Clear uses context page 1 (`2/4`). Save uses context page 2 (`3/5`). Fixed Undo (`1/3`). | Read the generated name. |
| A project for a track | Context page 1 Project on Pads or Files (`2/4`). Turn knob 1 (`0/0`). P8 opens the project (`1/2`). | A new name needs keyboard text. |
| Load a sound onto a pad | Main Files (`1/3`). Knob 1 moves through entries (`0/0`). LEFT or RIGHT changes folders (`1/1`). Main Take or Load (`1/3`). | The file filter needs keyboard text. Count the target separately. |
| Set up a pad | Main Edit (`1/3`). LEFT and RIGHT change the page (`1/1`). Turn knobs 7, 5, 3 and 1 (`0/0`). Context page 1 Mode, Clear, or Gate (`2/4`). | P7/P8 answer a question. |
| Mute groups: the closed hat stops the open hat | Main Edit (`1/3`). Select TONE with LEFT or RIGHT (`1/1` per page). Turn knob 3 for the mute group (`0/0`). | Listen to the pad hits. |
| The pad panel in a small window | LEFT or RIGHT selects the edit page (`1/1`). Knobs 7, 5, 3 and 1 change values (`0/0`). | Read both knob banks at the bottom. |
| Effects: drive, reverb, delay | Main FX (`1/3`). LEFT or RIGHT selects a slot (`1/1`). Knobs 2–4 edit the slot (`0/0`). In pad edit FX, knobs 5 and 3 set the pad sends (`0/0`). | Listen to each send. |
| Undo a change to the pads | Fixed Undo (`1/3`). | A no-op Undo shows its message. |
| What is set on a pad | Main Pads (`1/3`). Hit a pad (`0/0`). Context page 1 Mode (`2/4`). | Read pad state and values. |
| Waves on the pads | Main Edit (`1/3`). On SOURCE, turn knobs 5 and 3 for pad region start and end (`0/0`). | Wave drawing is an observation. |
| One look for the screens | Main Pads, Files, or Chop (`1/3` each). | Read the current page and list in the knob strip. |
| Hide the pads | Pads context page 1 Grid (`2/4`). Other views use context page 2 Grid (`3/5`). | The hint shows the grid state. |
| Set up a pad with the X-Session Pro | Use the X-Session profile and its controls. | Other-device story. |
| Chop a break | Main Chop (`1/3`). Knobs 1 and 2 move cursor and zoom. Knobs 3–6 move bounds. Knob 7 sets threshold (`0/0`). Main Cut, Parts, Hits, or Assign (`1/3`). Context page 4 Hits back, Cut -, End -, or Order (`5/7`). | Select a slice before removing a bound. |
| Keyboard and controller pads on Chop | Hit playing pads to select or assign. LEFT and RIGHT select a slice (`1/1`). | Pad hits and releases are separate. |
| Select what plays | Chop context page 1 Play mode, Loop, or As pad (`2/4` each). PLAY and STOP control preview (`1/1`). | The playback bar is an observation. |
| Move the bounds of a slice | Turn knobs 3 and 4 for slice start and end (`0/0`). | Steps follow the wave view. |
| Bounds in edit mode | Chop context page 1 Edit (`2/4`). Turn knobs 3 and 4 for the slice. Turn knobs 5 and 6 for the region (`0/0`). | Read the bound indicator. |
| Assign the slices to the pads | Main Assign (`1/3`). Select targets. Press P14 Apply. P15 Menu gives access to Order on context page 4. | Count layer switches, target hits, and replacement answers separately. |
| Assign mode ignores the rest | Main Assign (`1/3`). Press P15 Menu. Try Cut from the main panel. | The refusal leaves the plan in place. |
| Swap pads | Pads main Take (`1/3`) or context page 2 Take (`3/5`). Use P1–P12 on the target panel or P13–P16 on the playing layer. | Count the target separately. |
| Select file, copy and swap targets without a hit sound | Files main Take or Pads main Copy or Take (`1/3`). Use P1–P12 on the target panel or P13–P16 on the playing layer. P7/P8 answer (`1/2`). | The target hit stays silent. |
| Controller commands and the next pad hit | Use Take (`1/3`), LEFT or RIGHT on Pads (`1/1`), then select a target pad. | Check input order and Gate release. |
| MIDI knobs | Main Settings (`1/3`). LEFT or RIGHT selects MIDI (`1/1` per tab). Knob 1 selects a row and knob 2 changes a setting (`0/0`). | Enter opens the layout review for saved maps with legacy roles. |
| Bind the controller directly on the screen | Use the action. Context page 2 Learn (`3/5`). Touch a control. P8 saves or P7 cancels (`1/2`). | Back closes learning. |
| Record sound | Hold RECORD for microphone (`1/1`). Context page 2 Output record (`3/5`). | Place recorded sounds using the playing layer. |
| Session | Context page 1 or 2 Save (`2/4` or `3/5`). Context page 2 Exit (`3/5`). P8 leaves or P7 stays (`1/2`). | Read the save state. |

## Set or delete a Chop bound with a pad cell

1. Turn bank 1 knob 1 to the required point.
2. On the main panel, press P9 Cut to add a slice start.
3. For the Chop region, open More. Press P7 Start or P8 End to use the
   cursor position. Press P9 Whole to clear the region.
4. To delete the selected slice start, open context page 4. Press P2
   `cut -`. To delete a manual slice end, press P3 `end -`.

The route from the main panel to context page 4 is More and three Next presses.
Press P8 to accept a visible question. Press P7 to cancel.

## Target routes

Start on the playing layer with the panel hidden. Select a source before Copy or
Take. In Files, select a file. In Chop, prepare one slice for each test.
Use P1–P12 on the target panel. Switch to the playing layer for P13–P16.
For Assign, press P14 Apply. P13 cancels. P15 switches between targets and commands.

The software counts below include PAD BANK switches and one target hit.
Each count gives **detected shortcut / reveal fallback**.
Count any Yes or No answer separately. For multiple targets, count each extra hit.

| Flow | Change 124 | Change 125, target 1–12 | Change 125, target 13–16 |
| --- | --- | --- | --- |
| Copy, Take or file in hand | 4 / 5 | 3 / 4 | 4 / 5 |
| Assign one target and apply | 7 / 9 | 4 / 5 | 6 / 8 |

For the first row, use main Copy, Take or Files Take. For Assign on change 124,
More and Commit follow the return to the command layer. In change 125, P14 applies.
Repeat with an occupied pad. Check the question and count P7 or P8 separately.
Record the actual device sequence. Physical press counts remain open.

## Open owner checks

Use [Live checks](LIVE-CHECKS.md) for the physical device, terminal composition,
and listening checks. The transport reference is `renders/92-command-panel.wav`.

# SMC-PAD paths for current stories

This table covers the headings in [User stories](USER-STORIES.md).
It describes the SMC-PAD profile and command panel for changes 124 to 127.
Raw MIDI tests check these paths. Physical press counts remain open.

## Count a command

The counts give **visible panel / hidden panel with reveal**. On verified USB,
F12 MIDI defaults to `Hold panel` (change 131). The panel shows while PAD BANK is
held, and a pad hit during the hold runs its cell. The hold counts as one press.
With `Off` or an unverified unit, the hidden count includes one PAD BANK press and
one hit to reveal the panel.

| Action | Panel visible | PAD BANK held | PAD BANK and a reveal hit |
| --- | --- | --- | --- |
| Action of the current view | 1 | 2 | 3 |
| Screens view cell | 2 | 3 | 4 |
| Action of another screen | 3 | 4 | 5 |
| Knob turn in the active bank | 0 | 0 | 0 |
| LEFT, RIGHT, PLAY, STOP or RECORD | 1 | 1 | 1 |

A reveal shows the actions of the current screen or mode. LEFT shows the screens
view. RIGHT shows the actions again. A screen hit opens that screen and shows its
actions. The bottom row is the control row on every view (change 134). It has
P1 No, Back or Cancel, P2 Undo, P3 Redo and P4 Yes or Apply. The P numbers are the numbers
printed on the device. With an SMC-PAD connection, the pad grid, the side panel,
messages and Help use the same numbers. The keyboard key shows in the pad corner.

A visible panel takes LEFT and RIGHT. Without a hold, LEFT and RIGHT change a
bank, a page or a folder.
Count KNOB BANK separately if a turn needs the other knob bank. Count target pad
hits separately. A visible modal question takes P1 (No) or P4 (Yes) without a
hold, from either device layer. No reveal hit is necessary.

After an SMC-PAD input, hints and questions name SMC-PAD controls. After a key, they
name keys. The line under the LEFT and RIGHT hint shows the active mode, its next
step and its cancel control. It stays after the PAD BANK release.

| User story | SMC-PAD route and command counts | Exception or limit |
| --- | --- | --- |
| Terms | Read the screen and pad labels (`0/0`). | Observation. |
| Two hands | Playing pads trigger sound. LEFT and RIGHT select banks (`1/1`). Knob 8 controls master in both banks (`0/0`). | Pad hits are separate. |
| SMC-PAD command panel | Enter with PAD BANK. Use an action of the current view (`1/3`). LEFT and a screens view cell (`2/4`). RIGHT returns to the actions. | The USB shortcut can omit the reveal hit. |
| Chop the sound of a pad | Select a pad. Pads Chop on P7 (`1/3`). Knob 1 moves the cursor (`0/0`). Chop Cut, Parts, Hits, Assign or Commit (`1/3`). PLAY previews (`1/1`). | Count target pads and answers separately. |
| Edit a pad | Select a pad. Pads Edit (`1/3`). With the panel hidden, LEFT or RIGHT changes the edit page (`1/1`). Turn knobs 7, 5, 3 and 1 for the page values, from top to bottom (`0/0`). | Back closes the edit. |
| Read modes and operation results | Pads Copy or Back (`1/3`). Screens Help (`2/4`). In Help, LEFT and RIGHT select tabs and P5/P6 move (`1/1`). | Badges, results, and current knob roles are observations. |
| Read pad playback | Hit a playing pad. Read PLAY (`0/0`). | Pad hits and releases are separate. |
| Your own keys | Edit `~/.config/ptsr/keymap.toml`. | Keyboard-only document edit. |
| The splash screen | Open a screen from the screens view (`2/4`). Play a pad in the playing layer. | Splash text is an observation. A silent hit keeps it open. |
| Play a beat | Hit playing pads. PLAY and STOP control transport (`1/1`). Screens A1, then Tap sets tempo (`3/5` for the first tap). | A Gate release belongs to its original pad. |
| Name a pad | Press `Ctrl+N` and type a name. | Keyboard-only text entry. |
| Copy a pad without a growing name | Pads Copy (`1/3`). Select a target on the panel. Pads Clear (`1/3`). Screens Save (`2/4`). Fixed Undo (`1/3`). | Read the generated name. |
| A project for a track | Screens Open (`2/4`). Turn knob 1 (`0/0`). P4 opens the project (`1/2`). | A new name needs keyboard text. |
| Load a sound onto a pad | Screens Files (`2/4`). Knob 1 moves through entries (`0/0`). With the panel hidden, LEFT or RIGHT changes folders (`1/1`). Files Take or Load (`1/3`). | The file filter needs keyboard text. Count the target separately. |
| Set up a pad | Pads Edit (`1/3`). With the panel hidden, LEFT and RIGHT change the page (`1/1`). Turn knobs 7, 5, 3 and 1 (`0/0`). Pad edit Mode, Clear, or Gate (`1/3`). | P11/P12 answer a question. |
| Mute groups: the closed hat stops the open hat | Pads Edit (`1/3`). With the panel hidden, select TONE with LEFT or RIGHT (`1/1` per page). Turn knob 3 for the mute group (`0/0`). | Listen to the pad hits. |
| The pad panel in a small window | With the panel hidden, LEFT or RIGHT selects the edit page (`1/1`). Knobs 7, 5, 3 and 1 change values (`0/0`). | Read bank 1 and the master line at the bottom. |
| Effects: drive, reverb, delay | Screens FX (`2/4`). With the panel hidden, LEFT or RIGHT selects a slot (`1/1`). Knobs 2–4 edit the slot (`0/0`). In pad edit FX, knobs 5 and 3 set the pad sends (`0/0`). | Listen to each send. |
| Undo a change to the pads | Fixed Undo (`1/3`). | A no-op Undo shows its message. |
| What is set on a pad | Screens Pads (`2/4`). Hit a pad (`0/0`). Pads Mode (`1/3`). | Read pad state and values. |
| Waves on the pads | Pads Edit (`1/3`). On SOURCE, turn knobs 5 and 3 for pad region start and end (`0/0`). | Wave drawing is an observation. |
| One look for the screens | Screens Pads, Files, or Chop (`2/4` each). | Read the current page and list in the knob strip. |
| Hide the pads | Pads Grid (`1/3`). From another screen, Screens Pads, then Grid (`3/5`). | The hint shows the grid state. |
| Set up a pad with the X-Session Pro | Use the X-Session profile and its controls. | Other-device story. |
| Chop a break | Screens Chop (`2/4`). Knobs 1 and 2 move cursor and zoom. Knobs 3–6 move bounds. Knob 7 sets threshold (`0/0`). Chop Cut, Parts, Hits, or Assign (`1/3`). Chop Edit, then Hits -, Cut - or End - (`2/4`). | Select a slice before removing a bound. |
| Keyboard and controller pads on Chop | Hit playing pads to select or assign. With the panel hidden, LEFT and RIGHT turn the slice page until a slice lies on a pad (`1/1`). Then they select a pad bank (`1/1`). | Pad hits and releases are separate. |
| Select what plays | Chop Listen, Loop, or As pad (`1/3` each). PLAY and STOP control preview (`1/1`). | The playback bar is an observation. |
| Move the bounds of a slice | Turn knobs 3 and 4 for slice start and end (`0/0`). | Steps follow the wave view. |
| Bounds in edit mode | Chop Edit (`1/3`). Turn knobs 3 and 4 for the slice. Turn knobs 5 and 6 for the region (`0/0`). | Read the bound indicator. |
| Assign the slices to the pads | Chop Assign (`1/3`). Select targets. Press P4 Apply. P3 Menu, then P9 Order switches the order (`2/4`). One Undo on Pads restores all targets (`1/3`). | Count layer switches, target hits, and replacement answers separately. |
| Assign mode ignores the rest | Chop Assign (`1/3`). Press P3 Menu. Try Cut from the Chop actions. | The refusal leaves the plan in place. |
| Swap pads | Pads Take (`1/3`). From another screen, Screens Pads, then Take (`3/5`). Release PAD BANK and hit any of the 16 pads. | Count the target separately. |
| Select file, copy and swap targets without a hit sound | Files Take or Pads Copy or Take (`1/3`). Release PAD BANK and hit any of the 16 pads. P1/P4 answer (`1/2`). | The target hit stays silent. |
| Controller commands and the next pad hit | Use Take (`1/3`). With the panel hidden, use LEFT or RIGHT on Pads (`1/1`), then select a target pad. | Check input order and Gate release. |
| MIDI knobs | Screens Setup (`2/4`). With the panel hidden, RIGHT selects the MIDI tab (`1/1` per tab). Knob 1 selects a row and knob 2 changes a setting (`0/0`). | Enter opens the layout review for saved maps with legacy roles. |
| Bind the controller directly on the screen | Use the action. Screens Learn (`2/4`). Touch a control. P4 saves or P1 cancels (`1/2`). | P1 with no choice leaves learning. |
| Record sound | Hold RECORD for microphone (`1/1`). Pads Output records the output (`1/3`). | Place recorded sounds using the playing layer. |
| Session | Screens Save (`2/4`). Screens Exit (`2/4`). P4 leaves or P1 stays (`1/2`). | Read the save state. |

## Set or delete a Chop bound with a pad cell

1. Turn bank 1 knob 1 to the required point.
2. In the Chop actions, press P5 `cut here` to add a slice start.
3. To trim the sound, turn knobs 5 and 6. To remove the trim, turn them to the ends.
4. Press P11 `edit slice` to open Chop edit.
5. To join the selected slice with the slice on its left, press P6 `join left` in Chop edit.
   To join it with the slice on its right, press P7 `join right`. A dim cell has no neighbour.

Press P4 to accept a visible question. Press P1 to cancel.

## Target routes

Start on the playing layer with the panel hidden. Select a source before Copy or
Take. In Files, select a file. In Chop, prepare one slice for each test.
Release PAD BANK and hit any of the 16 pads to place the item (change 134).
Hold PAD BANK for the commands: P1 cancels, P2 undoes, P3 opens the menu, and P4
applies an Assign plan. During Assign, P16 switches the order.

The software counts below include PAD BANK switches and one target hit.
Each count gives **detected shortcut / reveal fallback**.
Count any Yes or No answer separately. For multiple targets, count each extra hit.

| Flow | Change 124 | Change 125, target 1–12 | Change 125, target 13–16 | Change 134, any target |
| --- | --- | --- | --- | --- |
| Copy, Take or file in hand | 4 / 5 | 3 / 4 | 4 / 5 | 3 / 4 |
| Assign one target and apply | 7 / 9 | 4 / 5 | 6 / 8 | 4 / 5 |

For the first row, use Pads Copy, Pads Take or Files Take. For Assign on change 124,
More and Commit follow the return to the command layer. From change 125, Apply on
the bottom row applies. Change 127 moves Apply to P4 and keeps the counts.
Change 134 makes all 16 pads targets on the playing layer.
Repeat with an occupied pad. Check the question and count P1 or P4 separately.
Record the actual device sequence. Physical press counts remain open.

## Open owner checks

Use [Live checks](LIVE-CHECKS.md) for the physical device, terminal composition,
and listening checks. The transport reference is `renders/92-command-panel.wav`.

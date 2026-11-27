# SMC-PAD paths for current stories

This table covers the headings in [User stories](USER-STORIES.md).
It describes the SMC-PAD profile and command panel for changes 124 to 143.
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
P1 No, Back or Cancel, P2 Undo, P3 Redo and P4 Yes or Apply. P1 first leaves an active
mode, such as erase or a copy in hand, and keeps the screen (change 138). The P numbers are the numbers
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
| Chop the sound of a pad | Select a pad. Pads `chop pad` on P7 (`1/3`). Knob 1 moves the cursor (`0/0`). Chop `cut here`, `cut by hits`, `equal parts`, `put on pads` or `fill pads` (`1/3`). PLAY previews (`1/1`). | Count target pads and answers separately. |
| Edit a pad | Select a pad. Pads `edit pad` on P5 (`1/3`). A page cell, P9 `tone page` to P13 `fx page`, opens its page (`1/3`). With the panel hidden, LEFT or RIGHT changes the edit page, around from the last to the first (`1/1`). Turn knobs 7, 5, 3 and 1 for the page values, from top to bottom (`0/0`). | Back closes the edit. |
| Read modes and operation results | Pads Copy or Back (`1/3`). Screens Help (`2/4`). In Help, LEFT and RIGHT select tabs and P5/P6 move (`1/1`). | Badges, results, and current knob roles are observations. |
| Read pad playback | Hit a playing pad. Read PLAY (`0/0`). | Pad hits and releases are separate. |
| Your own keys | Edit `~/.config/ptsr/keymap.toml`. | Keyboard-only document edit. |
| The splash screen | Open a screen from the screens view (`2/4`). Play a pad in the playing layer. | Splash text is an observation. A silent hit keeps it open. |
| Play a beat | Hit playing pads. PLAY and STOP control transport (`1/1`). Hold PLAY for the metronome (`1/1`). Screens A1, then `tap tempo` on P8 sets tempo (`3/5` for the first tap). | A Gate release belongs to its original pad. |
| Name a pad | Press `Ctrl+N` and type a name. | Keyboard-only text entry. |
| Copy a pad without a growing name | Pads `copy pad` on P6 (`1/3`). Hit a target pad. Pads `clear pad` on P8 and P4 yes (`2/4`). Screens Save (`2/4`). Fixed Undo (`1/3`). | Read the generated name. |
| A project for a track | Screens Open (`2/4`). Turn knob 1 (`0/0`). P4 opens the project (`1/2`). With no project, P4 `new project` makes a project with a date name (`1/2`). | A typed name needs keyboard text. |
| Load a sound onto a pad | Screens Files (`2/4`). Knob 1 moves through entries (`0/0`). With the panel hidden, LEFT or RIGHT changes folders (`1/1`). Files `load to pad` on P5 or `take file` on P6 (`1/3`). | The file filter needs keyboard text. Count the target separately. |
| Set up a pad | Pads `edit pad` (`1/3`). A page cell or LEFT and RIGHT change the page (`1/1`). Turn knobs 7, 5, 3 and 1 (`0/0`). Pad edit `play mode`, `copy pad`, `pitch gate` or `reset pad` (`1/3`). | P1/P4 answer the reset question. |
| Mute groups: the closed hat stops the open hat | Pads Edit (`1/3`). With the panel hidden, select TONE with LEFT or RIGHT (`1/1` per page). Turn knob 3 for the mute group (`0/0`). | Listen to the pad hits. |
| The pad panel in a small window | With the panel hidden, LEFT or RIGHT selects the edit page (`1/1`). Knobs 7, 5, 3 and 1 change values (`0/0`). | Read bank 1 and the master line at the bottom. |
| Effects: drive, reverb, delay | Screens FX (`2/4`). With the panel hidden, LEFT or RIGHT selects a slot (`1/1`). Knobs 2–4 edit the slot (`0/0`). In pad edit, P13 `fx page` (`1/3`), then knobs 5 and 3 set the pad sends (`0/0`). | Listen to each send. |
| Undo a change to the pads | Fixed Undo (`1/3`). The notice says what came back. | Undo is grey with nothing to undo, and a press says «nothing to undo». |
| What is set on a pad | Screens Pads (`2/4`). Hit a pad (`0/0`). Pads `play mode` on P12 (`1/3`). The cell shows the mode. | Read pad state and values. |
| Waves on the pads | Pads Edit (`1/3`). On SOURCE, turn knobs 5 and 3 for pad region start and end (`0/0`). | Wave drawing is an observation. |
| One look for the screens | Screens Pads, Files, or Chop (`2/4` each). | Read the current page and list in the knob strip. |
| Hide the pads | Pads `hide pads` on P15 (`1/3`). From another screen, Screens Pads, then `hide pads` (`3/5`). | The hint shows the grid state. |
| Set up a pad with the X-Session Pro | Use the X-Session profile and its controls. | Other-device story. |
| Chop a break | Screens Chop (`2/4`). Knobs 1 and 2 move cursor and zoom. Knobs 3 and 4 move slice bounds, knobs 5 and 6 trim the sound. Knob 7 sets the hit level (`0/0`). Chop `cut here`, `cut by hits`, `fewer hits`, `equal parts`, `trim silence` or `remove all cuts` (`1/3`). Chop `edit slice`, then `join left` or `join right` (`2/4`). | Select a slice before a join. Replacing hand cuts asks first. |
| Keyboard and controller pads on Chop | Hit playing pads to select or assign. With the panel hidden, LEFT and RIGHT turn the slice page until a slice lies on a pad (`1/1`). Then they select a pad bank (`1/1`). | Pad hits and releases are separate. |
| Select what plays | Chop `play range` on P15 or `loop play` on P14 (`1/3` each). Slice edit `hear as pad` on P8 (`2/4`). The cells show their state. PLAY and STOP control preview (`1/1`). | The playback bar is an observation. |
| Move the bounds of a slice | Turn knobs 3 and 4 for slice start and end (`0/0`). | Steps follow the wave view. |
| Bounds in edit mode | Chop `edit slice` (`1/3`). Turn knobs 3 and 4 for the slice. Turn knobs 5 and 6 for the trim (`0/0`). Slice edit `norm` on P5 sets the sound gain of the slice pads (`1/3`). | Read the bound indicator. |
| Assign the slices to the pads | Chop `put on pads` on P9 (`1/3`). Select targets. Press P4 Apply. P16 shows the method, `by hand` or `in order`, and switches it (`1/3`). In order, Apply is grey until the start pad hit. One Undo on Pads restores all targets (`1/3`). | Count layer switches, target hits, and replacement answers separately. |
| Assign mode ignores the rest | Chop Assign (`1/3`). Press P3 Menu, then P14 `loop play` (`2/4`). | From the keyboard, try Alt+O, Alt+P or Ctrl+G in Assign (1 each). The hint names Apply and Back. The plan stays in place. |
| Swap pads | Pads Take (`1/3`). From another screen, Screens Pads, then Take (`3/5`). Release PAD BANK and hit any of the 16 pads. | Count the target separately. |
| Select file, copy and swap targets without a hit sound | Files Take or Pads Copy or Take (`1/3`). Release PAD BANK and hit any of the 16 pads. P1/P4 answer (`1/2`). | The target hit stays silent. |
| Controller commands and the next pad hit | Use Take (`1/3`). With the panel hidden, use LEFT or RIGHT on Pads (`1/1`), then select a target pad. | Check input order and Gate release. |
| MIDI knobs | Screens Setup (`2/4`). With the panel hidden, RIGHT selects the MIDI tab (`1/1` per tab). Knob 1 selects a row and knob 2 changes a setting (`0/0`). Setup `choose` on P5 acts as Enter (`1/3`). | Enter opens the layout review for saved maps with legacy roles. |
| Bind the controller directly on the screen | From the keyboard: F10 Learn or Enter on an F12 MIDI row. Touch a control. Enter saves (1). Esc or P1 cancels (1). | Keyboard-only binding change (change 138). This is the only planned exception to the controller goal. On an F12 MIDI binding row, `choose` says «Bindings: keyboard only (F10, Enter)». In a pad or bank run, press Enter after each control (1 per row). |
| Record sound | Hold RECORD for microphone (`1/1`). Pads `record output` on P14 records the output (`1/3`). See [Stop an output recording](#stop-an-output-recording). | Place recorded sounds using the playing layer. |
| Session | Screens Save (`2/4`). Screens Exit (`2/4`). P4 leaves or P1 stays (`1/2`). | Read the save state. |

## Set or delete a Chop bound with a pad cell

1. Turn bank 1 knob 1 to the required point.
2. In the Chop actions, press P5 `cut here` to add a slice start.
3. To trim the sound, turn knobs 5 and 6. To remove the trim, turn them to the ends.
   P12 `trim silence` moves the trim to the first and the last sound.
   P13 `remove all cuts` removes all cuts. It asks first when hand cuts would go.
4. Press P11 `edit slice` to open Chop edit.
5. To join the selected slice with the slice on its left, press P6 `join left` in Chop edit.
   To join it with the slice on its right, press P7 `join right`. A dim cell has no neighbour.

Press P4 to accept a visible question. Press P1 to cancel.

## Target routes

Start on the playing layer with the panel hidden. Select a source before Copy or
Take. In Files, select a file. In Chop, prepare one slice for each test.
Release PAD BANK and hit any of the 16 pads to place the item (change 134).
Hold PAD BANK for the commands: P1 cancels, P2 undoes, P3 opens the menu, and P4
applies an Assign plan. In the action menu, P3 redoes and P1 returns to the targets. During Assign, P16 shows and switches the method.

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

### Replace a copy with a file

Start on Pads with a copy in hand and a file selected in Files.
On SMC-PAD, start on the second layer with the target panel visible.

| Input | Steps | Presses |
| --- | --- | --- |
| Keyboard | Press F6. Press Enter. Hit an empty pad. | 3 |
| SMC-PAD | Press P3 Menu, LEFT and P14 Files. Press P3 Menu again. Press P6 Take. Press PAD BANK to use the playing layer. Hit an empty pad. | 7 |

The badge changes from COPY to IN HAND. The target gets the file.

## Pads Sound strip

| Action | Keyboard | Visible panel / reveal |
| --- | --- | --- |
| Switch Sound and Rhythm | Alt+9 | On Pads, P16 (`1/3`) |
| Pin or release the selected sample | Alt+0 | On Pad Edit, P16 (`1/3`) |

From the Pads actions, use P5 to open Pad Edit. Use P16 to pin the sample.
Use P1 to leave Pad Edit. The route takes three hits from the visible panel.

## First A1 recording

The SMC-PAD counts give visible panel / PAD BANK held / reveal.

| Action | Keyboard | SMC-PAD |
| --- | --- | --- |
| Open A1 | F5 (1) | Screens P16 A1 (2 / 3 / 4) |
| Start count-in | Ctrl+R (1) | Tap RECORD (1) |
| Cancel count-in on A1 | Esc or Ctrl+R (1) | Hold PAD BANK and press P1 (2). With Hold panel off or unavailable, press P1 in the second bank (1). Tap RECORD to cancel from either bank (1). |
| Finish a live pass | Ctrl+R (1) | Tap RECORD (1) |
| Play or reset the loop | Space (1) | PLAY to start, STOP twice while playing to reset (1 / 2) |
| Pause or resume A1 | Alt+Y or Alt+T (1) | STOP or PLAY (1) |
| Restart from the beginning while paused | Space (1) | STOP, then PLAY (2) |
| Set tempo from two taps | Alt+B twice (2) | Open the A1 action panel and press P8 Tap Tempo twice (2 after opening) |

While A1 records, use the playing pads. Count those hits separately.

## Select and edit an A1 step

The first two lines show the position, loop length, grid, and recording quantize.
The grid marks the selected time column. Playback has a separate mark.

| Action | Keyboard | SMC-PAD |
| --- | --- | --- |
| Move one step | Left/Right or J/L (1) | Turn E1 one detent |
| Move one bar in the overview | Up/Down or I/K (1) | Close the panel, then press LEFT or RIGHT (1 per bar) |
| Enter Step Edit at the selected step | F7 (1) | Hold PAD BANK, press P6 Edit Steps (2) |
| Select the previous or next hit | I/K or Up/Down (1) | Hold PAD BANK, press P14 or P15 (2) |
| Add or remove a hit | Press the pad key (1) | Release PAD BANK. Press the playing pad (1) |
| Undo the edit | Ctrl+Z (1) | Hold PAD BANK, press P2 Undo (2) |
| Return to the overview at the same step | Esc (1) | Hold PAD BANK, press P1 Back (2) |

During the same hold, P2 Undo and P1 Back each take one hit.
When the hold shortcut is off, use the second pad bank for commands.
The first command pad press shows a hidden panel. Press the command again.

## Stop an output recording

Start with an active output recording. Close the command panel before you press STOP on SMC-PAD.

| Starting state | Keyboard | SMC-PAD |
| --- | --- | --- |
| Preview and A1 are idle | Alt+Y (1) | STOP (1) |
| Preview plays and A1 is idle | Alt+Y twice (2) | STOP twice (2) |
| A1 plays and preview is idle | Alt+Y twice (2) | STOP twice (2) |
| Preview and A1 play | Alt+Y three times (3) | STOP three times (3) |

The first press stops the preview if a preview plays. The next press pauses A1 if A1 plays.
The last press saves the take. A press to save before A1 pauses takes effect after the pause.

## Stop all sound

| Action | Keyboard | SMC-PAD |
| --- | --- | --- |
| Stop all sound | Ctrl+Period (1) | With the panel open, STOP twice (2). From the playing layer with Hold panel, hold PAD BANK, then STOP twice (3). |

Release PAD BANK to cancel the first press. With the panel closed, use STOP for normal playback.

## Redo

| Action | Keyboard | SMC-PAD |
| --- | --- | --- |
| Redo an edit | Ctrl+Y (1) | P3 Redo (1 with the panel open) |
| Redo an Assign choice | Ctrl+Y (1) | P3 Menu, then P3 Redo (2 with the target panel open) |
| Return from the Assign action menu | Esc closes Assign | P1 Back (1) |

## Change the click level

| Action | Keyboard | SMC-PAD |
| --- | --- | --- |
| Open Audio settings from Pads | F12 (1). Use Tab if needed. | Hold PAD BANK, LEFT, P11 Setup (3). Release PAD BANK. Use LEFT or RIGHT if needed. |
| Select Click level from the first row | Down four times (4) | E1 turns (0 presses) |
| Change Click level by 5 percent | Left or Right (1 per step) | E2 turns (0 presses) |

## Select a root folder in Settings

Start on Paths with the root row selected.

| Action | Keyboard | SMC-PAD |
| --- | --- | --- |
| Open the folder picker | Enter (1) | P5 Choose (1 with the panel visible, 3 with reveal) |
| Move the selection | Up or Down (1 per step) | E1 turn (0 presses) |
| Enter a folder or go to its parent | Right or Left (1) | RIGHT or LEFT (1) |
| Save the current folder or cancel | Enter or Esc (1) | P4 or P1 (1) |

## Open owner checks

Use [Live checks](LIVE-CHECKS.md) for the physical device, terminal composition,
and listening checks. The transport reference is `renders/92-command-panel.wav`.

## Control the shared FX returns

| Action | Keyboard | SMC-PAD |
| --- | --- | --- |
| Open shared FX from Pads | F4 (1) | Hold PAD BANK, LEFT, P9 FX (3) |
| Select the other slot | Tab (1) | LEFT or RIGHT with the panel closed (1) |
| Change Return from the first row | Down three times, then Left or Right (4) | E5 turn (0 presses) |
| Mute or unmute the selected return | M (1) | Hold PAD BANK and press P13 Mute return (2) |
| Return to Pads | F2 (1) | Hold PAD BANK, LEFT, P13 Pads (3) |

## Shape a pad sound

| Action | Keyboard | SMC-PAD |
| --- | --- | --- |
| Open SOUND from TONE | Enter, Shift+Tab (2) | Hold PAD BANK, P5 Edit pad. Release. Hold PAD BANK, P14 Sound page (4). Release. |
| Set Pan, Fine tune, Velocity or Curve | Up or Down selects a row. Left or Right changes its value (1 per step). | Turn E7, E5, E3 or E1 (0 presses). |
| Tune one cent | Shift+Left or Shift+Right (1 per step) | Hold PAD BANK and turn E5 (1). With the hold shortcut off, turn E5 (0 presses). |
| Set Hold from SOUND | Tab twice to ENV, Down three times, Left or Right (6 for the first step). | Hold PAD BANK, P10 ENV page (2). Release. Turn E1. |
| Return to Pads | Esc (1) | Hold PAD BANK and press P1 Back (2). Release. |

# How to play, chop and set up: step-by-step stories

This document describes the actions of the program and the keys for them. Each story is a
goal and steps. First read the sections "Terms" and "Two hands".

## Terms

- Pad: a key of the left half: `1 2 3 4`, `Q W E R`, `A S D F`, `Z X C V`. Hit: press a
  pad and release it. A hit selects the pad. The arrows do not select a pad.
- Slice: a part of the sample between two cuts.
- Assign mode: the mode that sets the slice layout. The slice layout is the positions of
  the slices on the pads.
- Cursor: the white line on the wave and the `!` above it. Only you move the cursor.
- Playhead: the thin green line. It moves while the sound plays.
- Screens: `F2` pads, `F3` chop, `F4` FX, `F6` files, `F12` settings.
- `Esc` goes back one step.
- `F1` shows the help for the screen that is open.

Notes:

- A note `[21]`, `[30]` and so on is the number of the plan that the step came from. All
  steps work in the program. Plans with a note still wait for a live check by the owner.

## Two hands

1. The fingers of the left hand play the pads.
2. The left thumb holds `Ctrl` or `Alt` for the right-hand keys. It also presses `Space`.
3. The right hand moves and changes: `J` `L` move left and right, `I` `K` move up and
   down. The arrows do the same.
4. A short press moves in a small step. If you hold the key, the movement becomes faster.
5. At the bottom of the screen there are three hint lines. They do not jump. A hint names
   the key and the action. `[43]` The groups are in columns. The order is the same on all
   screens: first the commands of the screen, then BANK, REC and the screens. When the
   width is not sufficient, the program removes the screens group first. `F1` shows it.
6. `[43]` `PgUp` `PgDn` and `Ctrl+1…8` page the banks on all screens with pads: on the
   pads, in edit mode, on the chop screen, in the files.

## Planned SMC-PAD flows

The keyboard steps below use current keys. The SMC-PAD steps are targets for changes
91–93. The current build has no complete SMC-PAD profile or command panel.

### Chop the sound of a pad

**Keyboard:** Hit pad `1` to select it. Press `F3` to open Chop with its sample.
Use `J` and `L` for the cursor and `I` and `K` for zoom. Press `M` to cut, `Y` for equal
parts, or `H` for cuts at hits. Press `P`, hit the target pads, and press `Enter` to
apply the layout. Press `Space` to listen. Press `F2` or `Esc` to return to Pads.

**SMC-PAD:** Hit the pad. Hold STOP and hit the Chop cell in the top panel row.
Turn knob 1 to move the cursor. The knob strip names zoom, hit threshold, and slice
bounds on the other knobs. Hold STOP and hit a panel cell to cut, make equal parts,
or cut at hits. Use the Assign cell, hit the target pads, and use Assign again to apply.
Press PLAY to listen. Tap STOP to return to Pads.

**Done when:** The slices are on pads and play without a keyboard action.

### Edit a pad

**Keyboard:** Hit a pad and press `Enter` to open its inspector edit. Use `Tab` and
`Shift+Tab` to select TONE, ENV, FILTER, SOURCE, or FX. Use `I` and `K` to select a
parameter. Use `J` and `L` to change its value. Hold Shift for a fine step. Press
`Enter` or `Esc` to close the edit.

**SMC-PAD, quick:** Hit the pad. Switch to knob bank 2. Turn its volume, pan, pitch,
cutoff, decay, and send knobs without opening the edit.

**SMC-PAD, full:** Hit the pad. Hold STOP and hit the Edit panel cell. Turn knob 1 to
select a parameter. Knobs 2–8 change the parameters named on the current page of the
knob strip. Press LEFT or RIGHT to change pages. Tap STOP to close the edit.

**Done when:** Every pad parameter can change from the SMC-PAD in at most two presses
plus knob turns.

## Read modes and operation results

`[87]` These steps use the default keys. Use the replacement shown if you changed a key.
Separate manual checks are in [Live checks, section 87](LIVE-CHECKS.md#87-visual-feedback-and-screen-catalogue).

1. On Pads, select a loaded pad. Press `Ctrl+C`. The yellow `COPY` badge identifies the pending operation.
   Empty targets show `◎`. Occupied targets show `!` because replacement needs confirmation.
   Press `Esc` to cancel. Swap uses `SWAP` and `◎` without another replacement question.
2. Open an existing replacement or clear question. Read `ASK`, its target and the shown answer controls.
   Controller answers appear only for connected bindings that can answer this question. Use the keyboard if no controller answer appears.
3. Read transient results above the help band. Info uses `·`, success uses `✓`, a warning uses `!`, and an error uses `✗`.
   Info and success last three visible seconds. Warnings last six. Manual save success lasts five.
   A new action acknowledges a shown Error without losing that action. Releases and repeated knob values do not acknowledge it.
4. Change a pad value with `L` or an existing knob. The upper-right panel names the original object, parameter and applied value.
   Numeric values have a range bar. The panel lasts one elapsed second. Pickup waiting, refusals and unchanged values do not renew it.
   Editors can hide it, but its clock continues. Settings choices describe configuration, not an unconfirmed sound-server result.
5. Read the top status on every screen. `KB` and connected MIDI inputs identify the sources.
   An arrow marks a new keyboard press or actual controller activity for one second. Compact `M1`, `M2` labels distinguish connected devices.
   Recording shows elapsed seconds. Narrow windows retain save and sound warnings before optional project, audio and input details.
   `Save failed` remains until a manual save succeeds. Dismissing its notice does not permit automatic retries.
6. Press `F1`. Help opens first. Press `Tab` for Messages. The latest 50 results show their levels and origins, newest first.
   `Tab` and `Shift+Tab` switch pages. `Up` and `Down` scroll Messages. `PageUp` and `PageDown` scroll one page.
   `F1` or `Esc` closes help. Local navigation does not change the parameter behind it. History ends when the process closes.

Yellow means that pads do more than normal playback. Marking still sounds. Existing MIDI routes stay unchanged during keyboard text entry.
The local ANSI examples are in `renders/screens/`. Run `task screens` to generate and check them without opening sound or MIDI devices.
These examples do not prove real kitty composition, controller operation or listening results.

## Read pad playback

1. Open `F2`. Next to `F2 PADS`, PLAY is dim when no pad plays.
2. Hit a keyboard pad or a controller pad. While a real pad voice plays, PLAY is blue
   with `▶`. A brief pad flash or a queued hit alone does not light it.
3. Change bank, open pad edit with `Enter` or hide the grid with `Ctrl+O`. PLAY counts
   all sounding pads in all banks, not only the selected pad or the bank in view.
4. When the last pad voice ends, PLAY becomes dim. A gate stays active through its
   release fade. A sample preview or an effect tail alone does not keep PLAY lit.
5. PLAY keeps the same width on start and stop. An existing MARK plate stays in place
   and keeps its yellow color. PLAY is an indicator, not a new transport control.

## Your own keys

1. `[64]` Create the file `~/.config/ptsr/keymap.toml`. Write in it only the keys
   that you want to change. The sections are the same as in `config/keymap.toml` of the
   project: `[global]`, `[edit]`, `[screens.pads]` and others.
2. `[64]` The line `"Ctrl+G" = "pad.mark_bank"` in `[screens.pads]` adds a key. Then
   `Ctrl+G` and `Ctrl+A` both mark the bank. `[76]` The old name `pad.scope_next` still
   works and does the same.
3. `[64]` The line `"Ctrl+A" = "none"` removes a key. Then `Ctrl+A` on the pads does
   nothing.
4. `[64]` Restart the program. The start folder is not important. Your keys are on top of
   the built-in keys. New built-in keys come with an update.
5. `[64]` If the file has an error, its path and line number show at the bottom. All
   built-in keys work. The pad keys (`Q`, `Shift+Q` and others) do not become commands.

## The splash screen

1. `[73]` Start the program. The wave strip above the pad grid shows the splash: the
   logo, `ptsr - PT's SampleR`, the version of the build, and the loading line. The pads,
   the help line and the status line show as always.
2. `[73]` The loading line names what loads now: `loading the session, 2 samples`. When all
   samples are loaded, it says `ready`.
3. `[73]` If the session names a sample file that does not exist, the line names this file:
   `cannot load gone.wav`. The program starts without this sound.
4. `[73]` The splash stays until the first sound. Nothing blocks the keys meanwhile: `F3`
   opens the chop screen, `Enter` opens the pad edit, and the splash stays.
5. `[73]` Make a sound: hit a pad, play a note on the controller, or start a recording.
   The splash ends and the strip shows the wave from then on.

## Play a beat

1. Hit a pad. The program plays the sound of this pad.
2. Hold a pad. A pad in gate mode sounds while you hold the key.
   `[68]` The status shows `no key releases: gate pads play one-shot`. This means that the
   terminal does not send key releases, and gate plays as a one-shot hit. `F1` names the
   terminals that send releases: Ghostty, kitty, WezTerm, foot.
3. Press `Shift` and a pad together. The hit has full velocity.
4. Press `Space` on the pads screen. The program plays a test click.

## Name a pad

1. `[80]` Hit a pad with a sample. Press `Ctrl+N` and edit its name. Press `Enter` to
   save it or `Esc` to keep the old name. Keyboard pad keys type instead of playing while the line is open.
2. `[80]` Clear the line and press `Enter` to show the file name again. The name stays
   in the project after restart. A new sound gets its file name; a failed load keeps the
   previous name. Swapping pads moves the name with the sound.

## Copy a pad without a growing name

`[87a]` Use a disposable project for these steps. Keep pad edit and text input closed.
The default keys below stay unchanged. Use your replacement keys if you changed the map.

1. Select a loaded pad named `kick`. Press `Ctrl+C`. Hit an empty pad.
   The target becomes `kick (copy 1)`. The source stays unchanged. Placement makes no hit sound.
2. Copy `kick` to another empty pad. It becomes `kick (copy 2)`.
   Copy `kick (copy 1)` to a third target. It becomes `kick (copy 3)`, not a repeated marker.
3. Delete `kick (copy 2)` with `Backspace`. Copy `kick (copy 3)` to an empty pad.
   The new name is `kick (copy 2)`. Numbers identify free names, not the order of earlier copies.
   `Ctrl+Z` restores the target. Its number becomes available again.
4. Read the proposed copy name in the replacement question.
   The target's old name still counts as occupied. If only `kick` and `kick (copy 1)` exist,
   replacing the latter proposes `kick (copy 2)`. `Esc` keeps its old sound and the copy in hand.
5. Change banks before you place another copy. All eight banks share the same name search.
   Missing and loading pads also reserve their names. Names in another project do not reserve a number.
6. Keep a slice number in the base: `break 2` becomes `break 2 (copy 1)`.
   Old text such as `rec 2 2 2` stays literal: its first copy is `rec 2 2 2 (copy 1)`.
   A manual final marker has meaning too: `kick (copy 7)` belongs to the `kick` name family.
   Its next copy takes the smallest free number. The source name stays unchanged.
7. Save with `Ctrl+S` and restart. Names and settings remain.
   The next copy uses the names that actually exist after loading. Saving needs no new counter or sample rename.

Only a final marker written exactly as ` (copy N)` has this meaning.
`N` must use positive ASCII digits without a leading zero. `kick (copy 01)` and `kick (Copy 1)` remain literal text.
Full Unicode names determine collisions before the existing display clipping.
The copy keeps the name and settings captured when you pressed `Ctrl+C`.
A changed proposed name needs another confirmation. Cancellation reserves no number.

Use existing controller bindings for copy, banks, targets and undo. Use keyboard commands where no binding exists.
This change adds no controller assignment. Separate owner passes are in [section 87a](LIVE-CHECKS.md#87a-numbered-pad-copy-names).
Sound stays unchanged. The local listening reference is `renders/83a-pad-copy.wav`.

## A project for a track

1. `[11]` Press `Ctrl+P`. The project list opens.
2. `[11]` Type a name. Press `Enter`. The program makes the project folder in
   `~/ptsr/projects` and opens the project. The project name is in the status line.
3. `[11]` Press `Ctrl+P` again. `I` `K` select a project. `Enter` opens it. Each project
   has its own pads, cuts and volumes.
   `[79]` The old project's sounds fade out without a click. Empty pads are silent on the
   keyboard and MIDI controller. Hits while the project opens are silent and do not play
   later. When you return, its pads play their saved samples and slices with their settings.
4. `[11]` After a restart, the last project opens.
5. `[35]` While `samples` of a new project is empty, `F6` opens where you were. Mark the
   folder with sounds as a favorite (`Ctrl+B`). It is first in each folder. In its parent
   folder it is one line with a star.
   `[68]` If a file is selected and not a folder, `Ctrl+B` marks the folder of the file.
   The status tells you: `★ breaks: favorite (the folder of this file)`.
6. `[35]` In the project list, the open project has the mark `● open`. The keys of the
   list are at the bottom.
7. `[35]` The program keeps the bank. After a restart, the bank where you stopped opens.

## Load a sound onto a pad

1. Press `F6`. The file list opens.
2. Select a file with `I` and `K`. `L` opens a folder. `J` goes back from a folder.
   `[30]` Hold `K` or the down arrow. The selection moves down the list while you hold
   the key.
3. `[30]` The selected file sounds immediately: preview is on by default. `Space` sets the
   preview off and on.
   `[31]` The folder path is directly above the list. The wave of the file is in the strip
   above the pads.
   `[33]` While the file sounds, a green line moves along the wave. It shows the point that
   you hear now.
4. `[41]` Press `Enter`. The file goes "in hand". Hit a pad: the file goes onto it. A
   second `Enter` puts the file on the selected pad. `Esc` returns it to the list.
5. `[41]` Press `Ctrl+Enter`. The program puts the file directly on the selected pad. Then
   it closes the list.
6. Wait for the load message. The pad is ready.
7. `[66]` If you save a sample to this folder from a DAW while the program runs, leave
   `F6` and go back to it. The program reads the list again. The new file shows. The
   selection stays in its position.
8. `[66]` After `Enter`, the filter `/` shows above the list: `filter: sn · Esc clears`.
   `Esc` clears the filter. The next `Esc` closes the screen.
9. `[66]` The wave has a label with the name of its sound: `kick.wav · 0.33 s`.

## Set up a pad

1. Press `F2`. Hit a pad. The pad is selected.
2. Press `Enter`. Edit mode opens. The panel frame becomes magenta. The title shows `EDIT`
   and all pages: `TONE ENV FILTER SOURCE`. The current page has a highlight.
   `[48]` The parameters are in blocks under lines with the page name. The line of the
   current page is bright.
   `[43]` The panel shows all parameters at the same time. The page changes only the knob
   numbers.
3. Press `I` and `K`. The program selects a parameter. Pages: TONE is volume and pitch.
   ENV is attack, decay, release. The parameters of the page show a knob number.
   `[54]` Press `Tab`. The next page opens fully. Its first parameter is selected.
   `Shift+Tab` goes back to the previous page. After the last page, the first page opens
   again.
4. Press `L`. The value increases. `J` decreases it. The parameter has a highlight for
   one second.
5. Hit the pad with the left hand. You hear the result. Edit mode stays open.
6. Press `Enter` or `Esc`. The mode closes.
7. `[76]` The scope is the pads that the next change hits: the marked pads, or the
   selected pad if no pad has a mark. Press `Ctrl+A`. All 16 pads of the open bank get a
   mark, and marking starts. Press `Ctrl+A` again. The marks of this bank go away.
   To mark all 128 pads, press `Ctrl+A` on each bank. While marking is on, the top line
   shows the yellow plate `MARK · hit pads · 17 marked`. When marking is off and pads stay
   marked, it shows `17 MARKED · Esc drops`.
   `[68]` You can see the scope outside edit mode too. The hint at `M` shows
   `mark: on, hit pads · scope: 16 marked`.
8. Press `M`. Marking starts. A hit on a pad puts a mark on it or removes it. `[76]` A
   marked pad has a dark yellow fill and a yellow frame. In the bank row, a bank with marks
   shows its marked pads as yellow dots, also when the bank is not open. `Esc` removes all
   marks and stops marking. In edit mode, the first `Esc` closes the edit, and the marks
   stay.
9. Press `H`. The pad mode changes: one-shot or gate.
   `[76]` In edit mode, press `Backspace`. A question asks `Reset 17 pads to clean?` (or
   `Reset pad Q to clean?`). Press `Enter`. The pads of the scope become as freshly
   loaded: volume 0 dB, one-shot, the other settings at the defaults. The sound and its
   bounds stay. `Esc` keeps the pads as they are. `Ctrl+Z` brings back the settings of
   each pad. Outside edit mode, `Backspace` takes the sound off the selected pad. If pads have
   marks, it first asks `Clear 3 pads?`. `Enter` takes the sounds off the marked pads, the
   files on disk stay. `Esc` keeps them. `Ctrl+Z` brings the sounds back.
10. `[83a]` `U` and `O` turn the banks, with the edit open or closed; `J` `L` and the
    arrows change the selected value instead. `[43]` `PgUp` `PgDn` and `Ctrl+1…8` also
    turn the banks. `[83a]` `Ctrl+C` takes a copy of the selected loaded pad: the next
    pad hit places it (an occupied pad asks first), and the copy keeps every setting.
    `[87a]` Its name uses one numbered `(copy N)` marker. Refer to "Copy a pad without a growing name".
    `Ctrl+Z` brings the target back.
11. `[21]` Turn knobs 1–3 of the controller in edit mode. They change the parameters of
    the page.
12. `[21]` The strip above the grid shows the envelope of the pad as a line on the wave.
13. `[27]` The pad panel tells what the pad plays: `break.wav · all` (the full file),
    `break.wav · slice 3` (part 3, it follows the cuts), `break.wav · pad` (its own
    bounds).
14. `[27]` `[48]` The last page of edit mode is SOURCE: source, start, end, knobs 1–3.
    On the source, `J` `L` select `all`, the parts of the file, or `pad`. Move the start
    or the end of the part. Then the pad becomes `pad`, and the adjacent pads do not
    change. `Ctrl+Z` links the pad to the cuts again.
15. `[44]` The third page is FILTER (`[48]` SOURCE moved to the end): filter, cutoff,
    resonance, knobs 1–3. On the filter, `J` `L` select `off`, `low-pass` (it decreases
    the highs), `high-pass` (it removes the lows), `band` (it keeps a band). On the
    cutoff, a press moves by one semitone. A hold moves by one octave. On the resonance,
    the steps are 1 % and 10 %. A pad that sounds changes immediately and smoothly. On
    this page, the strip above the pads shows the filter curve. The lows are on the left
    and the highs are on the right. The hump at the cutoff is the resonance. Marked pads
    change together.

## Mute groups: the closed hat stops the open hat

1. `[70]` Select the pad of the open hat. Press `Enter`. Press `K` until `mute group` on
   the TONE page. Press `L`: group 1. Below the pitch is the line `mute    1`.
2. `[70]` Do the same for the closed hat. Or do the two pads at the same time: `M`, hits
   on the two pads, `M`, `Enter`, `mute group`, `L`. The group goes onto all marked pads.
3. `[70]` Hit the open hat, then the closed hat. The open hat stops without a click.
4. `[70]` Pads of other groups and pads without a group continue to sound. All banks use
   the same groups.
5. `[70]` `[77]` The line under the pad name shows the group: `⇢ +1 mute 2`. Knob 3 of the TONE page changes
   the group. `J` from `1` goes back to `off`. `Ctrl+Z` cancels a group change.
6. `[70]` The project keeps the group. An old project opens without groups.

## The pad panel in a small window

1. `[70]` If the window is lower than the panel, the blocks fold. Each block other than
   the open page becomes a title with an arrow: `─ SOURCE ▸ ──`. The blocks fold from the
   bottom up. When the window is higher, the blocks unfold.
2. `[70]` If the folded panel also does not fit, the blocks scroll with the selected
   parameter. `▲ 2 more` at the top and `▼ 3 more` at the bottom tell you that `I` `K`
   go higher or lower.
3. `[70]` The pad lines at the top are always visible. The second line,
   `scope   pad 1`, names the scope: the pads that the next edit changes.
4. `[70]` `Tab` and `Shift+Tab` page the blocks. `I` `K` (`↑` `↓`) move through the
   items. These keys work also when edit mode is closed. Only edit mode (`Enter`)
   changes values. In edit mode the panel has a frame. The closed panel shows
   `Enter: edit` in its first line.
5. `[70]` In a narrow window, the labels are shorter (`vol`, `pit`, `atk`). The values are
   full. The title shows only the open page: `TONE 1/5`.

## Effects: drive, reverb, delay

1. `[56]` Each pad has its own drive. There are also two shared slots, FX 1 and FX 2.
   Each slot contains reverb or delay. A pad sends its own part of the sound to each slot.
   In a new project, FX 1 is reverb and FX 2 is delay.
2. Select a pad. Open edit mode with `Enter` or left Cue (b7, note 58).
   Press `Tab` or left forward (b1, note 43) until the FX page.
   Knob 1 changes drive. Knob 2 sets the send to FX 1. Knob 3 sets the send to FX 2. The
   send line shows the effect in the slot: `send 1  20 %  reverb`. `J` `L` change by
   1 %. Hold them for 10 %.
3. At 0 % the pad sounds as before. Drive adds dirt, and the volume almost does not
   change.
4. `[82]` Press `F4`. Both shared effect blocks are visible as compact cards with three
   parameter rows. They are beside each other when they fit, or stacked in a narrow
   window. Cards do not expand to fill empty space. The selected block has a different frame. These settings change every pad that sends to that block. Knob 1
   selects reverb or delay. For reverb, knobs 2 and 3 change size and tone. For delay,
   they change time (10–2000 ms) and feedback (up to 90 %). The echo decreases to silence.
5. Press `Tab` or `Shift+Tab` to select the other block. The same three page knobs now
   control that block. Selection alone does not change the sound. `I` and `K` select a
   parameter. `J` and `L` change it. Pads remain playable in either keyboard layout.
   To use two reverbs, select reverb in both blocks.
6. When you change the slot type, the old tail stops smoothly, without a click. If you
   set the old type again, its settings are the same as before.
7. The reverb tail and the echo continue after the pad stops.
8. `[82]` The pad panel keeps drive and both sends. Shared effect settings are only on
   `F4`. `F2` returns to the pad panel with its previous parameter selected.
   `Ctrl+Z` undoes an effect change. Block selection does not add an undo step.
9. `[77]` A pad with the drive or a send shows `✱` at the end of its mode line.
10. The project keeps all of these settings. An old project opens without effects.
11. Listen to `renders/82-shared-fx-screen.wav`: reverb at 0.1 s, delay at 1.1 s,
    a time change at 2.2 s, undo at 3.4 s, and a larger room at 4.6 s.
    The earlier comparisons are `56-drive.wav`, `56-reverb.wav`, `56-delay.wav`
    and `56-two-slots.wav`.
12. `[82]` Existing learned page knobs also work on `F4`. Each block and each device
    uses its own jump, pickup or scale state. In pickup mode, the other block waits for
    its own value. `F10` assigns the three knobs and the block selector on this screen.
    `F1` and the bottom help show the controls and selected block.

## Undo a change to the pads

1. `[61]` Turn the pad volume with a knob, or press `L` a few times. Press `Ctrl+Z`. One
   press returns the volume. A knob turn or a series of presses on one parameter is one
   step.
2. If many pads have marks, the change goes to all of them. `Ctrl+Z` returns all of
   them. The line shows `undo: cutoff on 16 pads`.
3. You can undo these changes: volume, pitch, envelope, filter, drive and sends, pad
   mode. Also: a swap with `Ctrl+X`, a clear with `Backspace`, a file on a pad, the
   source, the shared effect slots. `[76]` Also a reset with `Backspace` in edit mode.
4. You cannot undo the actions that change nothing: the pad selection, the bank, the
   pages, the marks. If you change the bank after a change, `Ctrl+Z`
   cancels the change. The bank stays.
5. If there is nothing to undo, the line shows `nothing to undo`. Undo keeps 100 steps.
   It keeps them until the program closes.
6. On the chop screen, `Ctrl+Z` has its own undo: the cuts and the layout.

## What is set on a pad

1. `[77]` Press `F2`. The line under the name of a pad is its mode line: the mode, the
   volume, the mute group and one mark: `⇢ +3 mute 2 ✱`. `⇢` is one-shot (press, and it
   goes on to the end), `⇥` is gate (it plays until you let go). The pad panel shows the mode as a word.
2. `✱` means that the pad has more settings: pitch, envelope, filter, drive or sends.
   Set the pitch to 0 again: `✱` disappears. The pad frame has no glyphs.
3. A slice is not a setting: after you chop, the pads with slices show no `✱`.
4. The volume, the mode and the group show on the line already, so they give no `✱`.
5. Select a pad. In the panel on the right, the changed values are bright. The other
   values are dim. A block with a change has a dot: `─ FILTER ● ──`.
6. `F1` explains `⇢`, `⇥`, `mute N` and `✱` in the section GLYPHS ON A PAD.

## Waves on the pads

1. Press `F2`. Each pad with a sound shows a wave below its name. The wave shows what the
   pad plays. The attack lifts the start. The decay lowers the wave to the end.
2. An empty pad shows a flat line.
3. Increase the attack or the decay in edit mode (refer to "Set up a pad"). The wave of
   the selected pad changes. With a long attack it rises more slowly.
4. Make the terminal lower. The waves get fewer lines: three, then two, then one. At a
   height of 26 lines, the pads have no waves.
5. The wave of each pad is on the pad. There is nothing below the panel on the right.
6. `[31]` The pads with waves are the same on `F2`, `F6` and `F3`.

## One look for the screens

1. `[31]` Change between `F2`, `F6`, `F3`. The wave strip at the top and the pads below it
   do not move. Only the right column changes.
2. `[31]` In the strip: on `F2` the output sound, on `F6` the selected file, on `F3` the
   sample with cuts.
3. `[31]` On the right: on `F2` the L and R levels and the pad panel. On `F6` the path
   and the file list. On `F3` the slice line and the hints.
4. `[31]` The strip uses all the height that the pads do not use. The minimum is 8 lines.
   Make the terminal lower. First the waves on the pads disappear. Then the strip becomes
   lower, to a minimum of 3 lines.

## Hide the pads

1. `[32]` Press `Ctrl+O` on `F2`, `F6` or `F3`. The pad grid disappears. The wave uses
   its position on the left, from the title to the bottom. The right column stays full.
2. `[32]` `[62]` Below the wave there is one line. It shows all banks, as above the grid,
   with the open bank in brackets. Then it shows the selected pad:
   `bank [1 ⢀⠂] 2⠑ 3⣿ … 8⠀ · pad W kick`. In a narrow window, the pad part becomes
   shorter first. Hit a pad. The line names it and flashes.
3. `[62]` In the bank line, the dot of the pad that you hit flashes. Half of the bank
   glyph becomes white. The same occurs above the grid when the pads are visible.
4. `[32]` On `F6`, the line shows where `Enter` puts the file.
5. `[32]` On `F3`, open assign mode (`P`). The grid shows again while assign mode is open.
6. `[32]` Hold `Ctrl`. The hint shows `O pads: hidden`. `Ctrl+O` shows the pads again.
7. `[32]` The program keeps the choice. After a restart, the pads stay hidden.

## Set up a pad with the X-Session Pro

`[82]` These instructions use the replacement profile. The program selects this profile
when the MIDI port name contains `X-Session`. There is no manual profile selector.
The table describes the defaults, not a hardware verification result.

### Install the replacement bindings

An existing user map stays unchanged until you confirm a reset. A program update or
restart does not replace it. This includes old destructive assignments for cut deletion,
layout application and replacement confirmation. Old Play assignments also remain until reset.

1. Stop the running program.
2. From the repository root, run `./target/release/ptsr`. Connect the X-Session Pro.
   The program reads device profiles from `config/devices` in the current directory.
3. Press `F12`. Open the MIDI tab.
4. Select the `reset to profile` row for `X-Session Pro`.
   This selects the device to reset, not a profile.
5. Press `Enter` to open the reset question.
6. Read the question. Press `Enter` again to install the replacement bindings.
   This deletes your custom bindings only for this device. Other devices keep their bindings.
   Press `Esc` instead to keep the existing map and custom bindings.

On first setup, the program copies the profile defaults into the user map automatically.
You do not need a reset for a device without an existing map.

### Physical controls

All messages use MIDI channel 1. CC means MIDI Control Change. The IDs are the names
shown in the MIDI settings. Button releases do not repeat commands.
The edit roles replace the screen roles while edit mode is open, except for Play preview.

| Physical control | ID | MIDI | Default role |
|---|---|---|---|
| Left Headphone | b2 | Note 44 | Open Files |
| Right Headphone | b3 | Note 45 | Open FX |
| Left backward | b4 | Note 46 | Pads: previous pad. Files: previous file. Chop: previous slice. Edit: previous page. FX: previous block. |
| Left forward | b1 | Note 43 | Pads: next pad. Files: next file. Chop: next slice. Edit: next page. FX: next block. |
| Right backward | b5 | Note 56 | Previous bank |
| Right forward | b6 | Note 57 | Next bank |
| Left Cue | b7 | Note 58 | Pads: open edit. Files: hold file. Chop: cut. Edit: close. |
| Right Cue | b8 | Note 59 | Back. Files: open Chop. Chop: open slice edit. Edit: close. |
| Left Play | b10 | Note 70 | Files: preview on/off. Chop: play/stop. These roles also work in edit. Unassigned elsewhere. |
| Right Play | b9 | Note 69 | Files: preview on/off. Chop: play/stop. These roles also work in edit. Unassigned elsewhere. |
| Left numbered knob 1 | k1 | CC24 | Pads: volume. Chop: slice start. Edit / FX: parameter 1. Unassigned elsewhere. |
| Left numbered knob 2 | k2 | CC25 | Chop: slice end. Edit / FX: parameter 2. Unassigned elsewhere. |
| Left numbered knob 3 | k3 | CC26 | Chop: cursor. Edit / FX: parameter 3. Unassigned elsewhere. |
| Left Hi | k4 | CC27 | Chop: region start. Unassigned elsewhere. |
| Left Mid | k5 | CC28 | Chop: region end. Unassigned elsewhere. |
| Left Low | k6 | CC29 | Unassigned |
| Right numbered knob 1 | k7 | CC31 | Unassigned |
| Right numbered knob 2 | k8 | CC32 | Unassigned |
| Right numbered knob 3 | k9 | CC33 | Unassigned |
| Right Hi | k10 | CC34 | Unassigned |
| Right Mid | k11 | CC35 | Chop: threshold. Unassigned elsewhere. |
| Right Low | k12 | CC36 | Chop: zoom. Unassigned elsewhere. |
| Left Volume fader | fader_b | CC11 | Selected pad volume on all screens |
| Left Pitch fader | fader_a | CC12 | Selected pad pitch on all screens |
| Right Volume fader | pitch_a | CC14 | Master volume on all screens |
| Right Pitch fader | pitch_b | CC15 | Unassigned |
| Crossfader | crossfade | CC17 | Unassigned |
| Crossfader inverse message | No ID | CC20 | Unassigned. Ignore this mirrored message. |

Cue only opens edit or takes a file. It does not apply a pending plan or place a held file.
No default button deletes a cut, applies a layout or confirms replacement.
Use the keyboard or explicit MIDI learn for these actions.

### Edit and chop

1. Select a pad. Press left Cue (b7, note 58). Pad edit opens.
2. Press left forward (b1, note 43) for the next page.
   Press left backward (b4, note 46) for the previous page.
   You do not need to select a parameter before you turn its knob.
   You can assign a page knob in `F12`, group PADS · EDIT, row "page (knob)".
   Each section of its travel selects one page immediately.
3. Turn left numbered knobs 1–3 (k1–k3). They change the parameters of the current page.
   In `jump` mode, the parameter follows the knob immediately. Refer to "MIDI knobs" for other modes.
   Attack, decay and release follow a curve. Near zero, the knob gives finer control.
4. Press either Cue to close edit. The controls return to their screen roles.
5. On Chop, press left Cue to cut. Press right Cue to open slice edit.
   Press either Cue again to close edit. Either Play controls preview, including inside slice edit.
6. Outside edit, k1 and k2 move slice bounds. k3 moves the cursor.
   Left Hi and Mid set the region start and end. Right Mid and Low change threshold and zoom.
7. Press `P` to start assignment. Press `Tab` to select sequential placement.
   Select the start pad.
   Press `Enter` to apply the plan. Confirm replacement if the program asks.
   For a controller-only layout, first learn a control as commit in `F12`.
   That learned control starts the plan, applies it and confirms replacement on successive presses.
   Neither Play button commits a layout in the replacement defaults.
8. Press right Headphone to open FX. Left backward / forward select the effect block.
   Left numbered knobs 1–3 change the selected block. Left Headphone opens Files.

## Chop a break

1. Put a sound on a pad. Press `F3`. The chop screen opens.
   `[42]` If the pad has the full file, `F3` starts a new chop, as in samplers.
   `[45]` The old chop stays with its pads. They sound as before and show their own
   `slice N`. `F3` on such a pad opens its chop. The title shows `chop 1 of 2`. A cut in
   this chop moves only its pads. `Ctrl+Z` immediately after a new chop returns the old
   chop. From the file list (`F6`, then `F3`), the newest chop of the file opens. A
   project that you saved before restores a lost chop from the bounds of its pads.
   `[37]` If the pad has a part, the wave opens on this part. You can see the adjacent
   parts at the edges.
   `[69]` It is the same for a pad where you moved the start or the end. The wave opens on
   its part. The slice where the part starts is selected. The cuts of the chop do not
   change.
   `K` zooms out to the full file. The bank with the full-file pad has the mark `◆`. This
   pad shows `◆ all`. `[46]` The frame of this pad is dashed. The frames of pads with
   parts are solid.
   `[38]` The wave color shows the band that is louder than usual for this file. The low
   (kick, bass) is red. The mid (voice) is green. The high (hat, cymbals) is blue.
   `[40]` Only the selected part has color. The other parts are grey at the same
   brightness. The color shows your position.
2. Press `Space`. The program plays from the cursor. Press `Space` again. The program
   stops. The cursor stays in its position.
   `[68]` While the sound plays, the title shows `LISTEN ▶`.
3. Press `M` while the sound plays. A cut goes to the point of the sound now. When the
   sound does not play, `M` puts a cut at the cursor.
4. Press `J` and `L`. The cursor moves by approximately one cell of the screen. If you
   hold the key, the cursor becomes faster. For a smaller step, zoom in on the wave.
   `[21]` When the cursor goes into a different slice, the slice has a highlight and
   plays one time.
5. `[43]` Press `N` two times. Then the cursor moves from cut to cut. `L` puts it on the
   next cut or at the end. `J` puts it on the previous cut or at the start. The slice
   where the cursor stops plays. Here `PgUp` `PgDn` page the banks, as on all screens.
6. Press `I`. The wave zooms in around the cursor. `K` zooms out.
7. Press `N`. The cursor step changes: equal steps, from hit to hit, from cut to cut.
    `[51]` In hit mode, marks `▾` are above the wave. The cursor stops only on them. They
    are the hits that are a maximum of 12 dB quieter than the loudest hit. The program
    ignores small rises in a sound. After `H`, the marks are exactly on its cuts.
    The two use the same threshold.
    `[04d]` When the mode "from hit to hit" is on, `M` puts the cut on the nearest hit.
    The maximum distance is 90 ms from the point where you pressed. The status shows
    "marker snapped to hit". Set `N` to a different mode: the cut goes where you pressed.
8. Press `Y`. The program chops into equal parts: 2, 4, 8, 16, 32.
    `[83a]` Press `H`. The program chops by hits. The first press puts cuts on the
    loudest hits. Each next press decreases the threshold: −12, −18, −24, −30 dB, then −6
    again. `Shift+H` steps the threshold one value back, stopping at −6 dB without
    wrapping. Quieter hits get their own cuts. The status names the threshold and the number
    of cuts. `Ctrl+Z` returns the cuts of the previous press.
    `[83a]` Press `Ctrl+G`. The pad this chop came from normalizes to a −1 dB peak.
    `[83a]` While assigning, `Tab` switches the plan between in order and by hand.
    `[68]` `H` replaces manual cuts, and the status tells you:
    `by hits at -6 dB: 3 slices (were 16) · Ctrl+Z brings them back`.
    `[04d]` Knob k11 of the X-Session changes the same threshold continuously. The range
    is from 0 to −40 dB below the loudest hit. Cuts show and disappear immediately with
    the knob. The knob catches the threshold as the knob mode sets (refer to "MIDI
    knobs"). One turn is one `Ctrl+Z`.
9. Press `,`. The start of the region goes to the cursor. `.` sets the end. `Ctrl+,`
   returns the full length.
   `[35]` If the slices are already on pads, the pads outside the region continue to play
   their sound. They become `pad`. `Ctrl+Z` returns the region and links them again.
    `[27]` The title shows "cuts of break.wav". The cuts belong to the file.
    `[36]` Next to the part number there is a mark. It is `●` in the bank color if the
    part is on a pad. It is a dim `○` if the part is on no pad. If there is sufficient
    space, the pads follow the mark: `3●2-4` is part 3 on bank 2, key 4. The part line
    names its pads: `on 1-E, 2-W`.
    `[36]` If you put a new cut in a part that is on a pad, the pad plays the left
    piece, from the start of the part to the new cut. The pad stays linked. The right
    piece is a new part `○`. The status shows `pad 2-Q is shorter now`. `Ctrl+Z` returns
    the full part.
    `[36]` At the bottom are your real pads of the bank, as on `F2`. A hit plays the pad
    and selects its part. The chop screen opens on the bank that has the parts.
    `Ctrl+1…8` page the banks. Pads without parts of this file are dim. To hear a part
    without a pad, use the cursor: `J` `L`.
    `[43]` `PgUp` `PgDn` also page the banks.
    `[04e]` `P` starts assign mode from the part that you heard last.
    `[04e]` The program keeps the cuts also when no part is on a pad.
10. Press `Backspace`. The cut on the left of the cursor disappears. The slice joins the
    previous slice.
    `[43]` In the hint, this is `Bksp remove cut`.
    `[21]` If the cursor is exactly on a cut, this cut disappears. Hold `Ctrl` or `Alt`.
    The bound has a highlight. `Ctrl+Backspace` removes the left bound. `Alt+Backspace`
    removes the right bound.
11. Press `Ctrl+Z`. The program cancels the last action. `[35]` The slice below the
    cursor becomes the selected slice. Edit mode and the bounds work with it. The status
    line tells how many pads returned.
12. `[35]` The hint line above the help tells the next step: chop, assign, move the
    bounds.

## Keyboard and controller pads on Chop

`[84]` Use the same pad positions in the current bank from either input. A device
without a profile uses notes 36–51. A control explicitly bound to `pad.1`…`pad.16`
also works. The X-Session Pro has no physical pad grid; its default command buttons
are not pad hits. The SMC-PAD profile is still a template, not a verified device map.

### Keyboard pass

1. Open a cut sample on `F3` before any slice is linked to a pad. Hit a pad key.
   It selects and previews the slice at that position on the current bank page.
   `PgDn` opens the next page of slices.
2. Put slices on pads with `P`, target hits and `Enter`. In listen mode, hit a linked
   pad. It plays with that pad's pitch, filter and other settings and selects its slice.
   `Shift` gives full velocity; a plain key gives velocity 80. In listen and slice edit,
   the thin green line follows that pad's real playback position and pitch. It starts
   again on a new hit and disappears when the voice ends. Playback progress does not
   move your edit cursor. `Space` and `M` keep their existing behavior.
3. Press `P` with a slice selected. Each target hit adds that slice to the by-hand
   plan, previews it and selects the next unplanned slice. Turn banks to add targets
   there. Hits do not apply the plan. `Enter` applies; an occupied target asks first.
4. Under a question or in `F10` learn, new pad hits stay silent and do not change a
   plan. A gate key held before learn still releases its original pad, even after a
   bank or screen change. A new silent learn key does not stop an older held pad.

### Controller pass

1. Repeat the listen and assign steps with controller notes or learned pad buttons
   instead of keyboard pad hits. Use the keyboard for commands that have no binding.
   The slice selection, plan and target must match the keyboard pass.
2. A linked pad keeps the note velocity or the CC press value. A light hit is quieter.
   Compare full MIDI velocity 127 with `Shift` on the keyboard. Bare slice previews
   keep their existing preview level; they do not gain a new velocity rule.
3. Release the original note or button after a bank or screen change, question or
   `F10`. The original gate pad stops. Disconnecting that device also releases it.
   New hits in learn stay silent and cannot take another control's held pad.

The same green line works with Braille and picture waves. If several pads of this chop
sound, it follows the most recently started voice that still sounds. While a Space or
slice preview sounds, its existing line has priority. A pad of another sample does not
put a line on this waveform. The line does not scroll the view to follow the sound.
The LISTEN badge keeps the same width: dim at rest, blue with `▶` while a preview or
linked pad of this chop plays. A voice beyond the visible wave still lights it. The
file name does not shift. An effect tail alone does not keep LISTEN lit.

Listen to `renders/84-input-parity.wav`: first three identical full-strength hits
(keyboard, note, learned button), then quiet/loud note hits and quiet/loud button hits.
These are offline tests, not physical-controller acceptance. The separate manual passes
are in `docs/LIVE-CHECKS.md`, section 84.

## Select what plays

1. On the chop screen, press `O`. The play mode changes.
2. The modes: to the end, to the cut, the full slice.
3. The green bar above the wave shows what `Space` plays.
4. Move the cursor. The bar immediately shows the new part.
5. `[49]` Press `Alt+O`: loop. The hint shows `loop: on` next to the mode. `Space` plays
   the selection again and again until you press `Space` again. Move a cut or a bound.
   The next loop plays the new positions. There is no click where the loop starts again.
   `Alt+O` also stops the loop. Each time the chop screen opens, the loop is off.
6. `[50]` Press `Alt+P`: "as the pad". The hint shows `as pad`. The part plays with the
   settings of the pad that has it (the first pad in sequence). The settings are pitch,
   volume, envelope and filter. A part without a pad plays raw. A hit on this pad does
   not stop the playback.

## Move the bounds of a slice

1. Put the cursor in a slice, or hit a pad with a slice.
2. Hold `Ctrl` with the left thumb. The left bound of the slice becomes amber.
3. Press `J` and `L`. The left bound moves.
4. Hold `Alt`. The right bound becomes amber. `J` and `L` move it.
5. A bound moves like the cursor. A short press is approximately one cell of the screen.
   If you hold the key, the movement becomes faster in a quarter of a second. At all
   zoom levels, the speed on the screen is the same.
6. The pads of the same file get the new part. The status line shows their number.

## Bounds in edit mode

1. On the chop screen, select a slice.
2. Press `Enter`. The BOUNDS page opens: the start and the end of the slice.
3. Press `I` or `K`. The program selects the start or the end. The selected bound is
   amber.
4. Press `J` and `L`. The program moves the selected bound.
5. Press `Enter` or `Esc`. The mode closes.

## Assign the slices to the pads

1. On the chop screen, press `P`. Assign mode starts from the selected slice.
2. Hit the keyboard or controller pads in sequence. Each hit adds the selected slice
   to the plan at that target, previews it and selects the next unplanned slice.
   No pad changes until you apply the plan.
   `[35]` `J` and `L` select the slice that you need and play it. A hit on a pad puts
   this slice. For example, "slice 2 on pad C" is `J` to slice 2 and a hit on C.
3. Press `Tab`. The mode changes to "in order". A hit on a pad puts all slices from this
   pad at the same time. `[67]` If you do not want to hit a pad, press `Enter`. It starts
   from the selected pad and shows the plan. A second `Enter` applies the plan.
4. Press `Ctrl+Z`. The program cancels the last step of assign mode.
5. Press `Enter`. The program applies the plan and closes assign mode.
6. If the pads are not empty, the program asks. `Enter` confirms the replacement. `Esc`
   cancels it.
7. Press `Esc`. Assign mode closes and does not apply the plan.

## Assign mode ignores the rest

1. Open assign mode. Press `M`, `I`, `K` or `Ctrl+J`.
2. The cuts, the zoom and the cursor do not move. The assign plan does not change. The
   status line tells you: `Enter` applies, `Esc` closes.
3. Only these work: the pads, `Tab`, the slice selection, undo, `Enter` and `Esc`.

## Swap pads

1. Hit a pad. Press `Ctrl+X`. The pad goes "in hand". The panel shows
   `in hand: pad 3 · amen 3`. `[35]` The envelope moves with the pad.
2. Hit a different pad. The two pads change positions.
3. Press `Esc`. The pad goes back, and there is no swap.

## Select file, copy and swap targets without a hit sound

`[84]` On Pads, a keyboard pad, MIDI note or learned pad button selects the same
single target in the current bank. A target hit does not also play that pad.

1. Take a copy with `Ctrl+C`. Turn to another bank and hit an empty pad. The copy
   goes there without sound. An occupied target asks first. Further pad hits do not
   change or confirm the question. `Esc` keeps the copy in hand; `Enter` replaces only
   the target named in the question.
2. Take a file with `F6` and `Enter`. Hit an occupied target in another bank. The
   selected pad is the target named in the question. `Esc` keeps its old sound and
   the file in hand. A target hit does not confirm replacement.
3. Take a pad with `Ctrl+X`, turn banks and hit a target. The pads swap without a hit
   sound, and the target stays selected. One `Ctrl+Z` on Pads restores both pads.
4. With a clear or replacement question open, new pad hits stay silent. A gate pad
   held before the question still stops when its original key or control is released,
   including after a bank or screen change. Closing the question returns normal play
   only if no file, copy or other target mode is still open. Marking still plays hits.

## Controller commands and the next pad hit

`[85]` Use the current controller bindings. This change adds no key or device profile.

1. On Files, select a file and switch preview off if you want to hear only pad sounds.
   Press the controller's take-file command, then hit a pad at once. Even before the
   screen updates, that hit selects a file target; it does not play the old sound.
   An empty target loads the file. An occupied target asks before replacement.
2. Hit a different occupied pad while that question is open. The first target and
   question stay. The new hit neither plays nor confirms replacement. `Esc` keeps
   both old sounds and the file in hand. Keyboard `Enter`, target hits and `Esc`
   keep the same behavior.
3. Cancel the held file, turn the bank and hit a pad. Commands keep their received
   order; the hit uses the bank and mode left by them. This also applies when the
   command comes from one controller and the pad from another. Each hit keeps the
   velocity of its original note or learned button.
4. Release a gate control while the screen is still catching up. Its original pad
   still receives the release, also for note-on zero, CC zero or device disconnect.
   A delayed new hit keeps its own release even if an older direct hit already stopped.
   Changing bank, screen, a question or F10 does not move that release to another pad.
5. Outside pending commands and target modes, controller hits still go straight to
   sound without waiting for the screen. The later screen update does not play them
   twice. Keyboard/MIDI Chop holds stay independent; opening Chop does not replay an
   old direct notification. Text, pickers and F12 learn keep their existing boundaries.

These paths have recorded-input and offline audio tests. They do not prove physical
controller operation or owner listening. Use the separate passes and render map in
`docs/LIVE-CHECKS.md`, section 85. The SMC-PAD map remains an incomplete template.

## MIDI knobs

1. Press `F12`. Change the tab with `Tab`. Open the MIDI tab.
2. The list is folded into groups: "PADS · 8 actions · 2 learned". The group of the
   screen where you opened the settings is unfolded. The other groups are folded. `PgUp`
   and `PgDn` go from group to group. `→` unfolds a group. `←` folds it.
3. Select an action with `I` and `K`. Press the learn key. Turn a knob of the controller.
   `[74]` The knob is not bound yet. The row shows it in amber next to the old control:
   `none → k5`. If you touch a different control, it replaces the first. `Enter` binds
   the knob and saves it. `Esc` keeps all as it is. The config file does not change.
4. `[74]` This example uses custom b9 bindings, not the replacement profile defaults.
   Before `Enter`, the row shows what the control changes. If the control has a
   binding in the same group, that row becomes magenta with `✗ loses b9`. `Enter` moves
   the control. If the control has a binding in a different group, a grey line shows it
   under the row: `b9 on GLOBAL: bank next · stays`. That binding stays. For example, b9
   switches the pad mode on the pads screen and goes to the next bank on the other
   screens.
5. In edit mode, knobs 1–3 change the parameters of the page. Learn them in the group
   "PADS · EDIT" or "CHOP · EDIT". One knob can change the volume on the pads and a
   parameter in edit mode. The layers are independent. Outside edit mode, an edit knob
   does nothing.
6. The row "pads 1–16" learns all pads in one pass. Press the learn key. Hit the
   controller pads in sequence. If a control already has a binding, the program ignores a
   hit on it. `Esc` stops the pass. The learned bindings stay. `Backspace` removes all
   sixteen at the same time. The row "banks 1–8" learns the bank selection by number in
   the same procedure.
7. `[58]` The first row of the tab is "knob response". It sets how a knob catches the
   value. `←` `→` or the learn key change the mode. The mode starts to work immediately,
   and the program keeps it:
   - `jump` (default): the parameter goes to the knob position immediately. If the knob is
     far from the value, the sound jumps at the first touch. There is no click.
   - `pickup`: the parameter waits until the knob gets to the value. The screen shows the
     direction to turn. You wait again on each new pad and page.
   - `scale`: the parameter changes from the first movement, in smaller steps than the
     knob. It gets to the knob value at the end stop.
   `PgDn` from this row goes to the first group.
8. `[59, 82]` On first setup, the program copies the X-Session Pro profile into your bindings.
   The MIDI port name must contain `X-Session`. The program selects the profile automatically.
   Left Cue (b7, note 58) opens pad edit. k1–k3 change the current page parameters.
   After that, all bindings are yours. The row shows the control name, such as `b7 (note 58)`.
   An existing map keeps its old assignments until a confirmed reset.
9. `[59]` Select a row. Press `Backspace`. The binding disappears fully. The profile does
   not return it, also after a restart.
10. `[59, 82]` Below "knob response", select `reset to profile` for `X-Session Pro`.
    Press `Enter` to open the question. Press `Enter` again to reset only this device.
    Reset deletes your custom bindings for this device. `Esc` keeps the existing map unchanged.
    Restart the newly built program before this reset. Refer to "Install the replacement bindings".
11. `[71]` Each bound knob and key does an action. At start, the program shows at the
    bottom each binding to an action that it does not have: file, key, name. The program
    removes such bindings from your config one time. Then it tells how many it removed.
    On the chop screen, knobs k6–k10 of the X-Session are now free. `Ctrl+K` is free on
    all screens.

## Bind the controller directly on the screen

1. `[47]` `[57]` On Pads, Chop, Files or FX, press `F10`.
   `[87]` The fixed bottom help band shows learning context, details and working controls. The top status remains visible.
   Context names the target layer and page, such as `PADS · EDIT · FILTER`.
2. `[52]` Select an item with its own key. A pad key selects a pad without a new sound.
   A command key selects its command without applying it. For example, `M` on Chop selects `cut`.
   On Pads, Chop or FX, use `I` and `K` to select parameters. Files has no parameter walk.
   The selected item stays lit. Its label shows its control or `none`.
3. `[57]` The second procedure: first do the action, then press `F10`. For example, open
   the pad edit, go to the cutoff and press `L`. Then press `F10` in less than 5 seconds.
   The mode opens directly on the cutoff: `cutoff · PADS · EDIT · FILTER`. If you press
   `F10` after 5 seconds, the mode opens with no selection.
4. `[57]` Move a knob or press a button of the controller. The control shows in amber
   next to the old control: `cc 24 → cc 26`. It shows in the panel line and on the pad.
   It is not bound yet. If you touch a different control, it replaces the first.
5. `[57]` `Enter` binds. With the first procedure, the selection goes to the next pad,
   parameter or command. Thus you learn pads 1–16 in sequence. With the second procedure,
   the mode closes. `Esc` keeps the old binding.
6. `[57]` A control assigned elsewhere on the same layer marks its losing item with `✗ leaves`.
   The learning block warns about that change. `Enter` moves the control. `Esc` keeps the current assignment.
   Other-layer assignments remain. The block shows that information without a conflict warning.
   A knob can control volume outside edit and cutoff inside edit.
7. `[57]` A parameter needs a knob or fader. A command needs a button or pad.
   `[87]` A refused parameter control shows `Use a knob or fader. A button cannot change this value.`
   A refused command control shows `Use a button or pad. A knob repeats this action.`
8. `[57]` `Backspace` shows `cc 28 → none`. `Enter` removes the binding. `Esc` keeps it.
   `F10` closes the mode.
9. `[57]` The binding goes into the layer where you made it. If edit mode was open, the
   binding works in edit mode. If not, it works on the screen.

## Record sound

1. `[83a]` Hold `Alt+R`. The program records the microphone. Release the keys. The program stops
   and saves the recording. `[11]` The file goes to `samples` of the open project. If no
   project is open, it goes to the samples root from the settings. The recording is "in
   hand" immediately: hit a pad.
2. Press `Ctrl+T`. The program records its own sound. Press `Ctrl+T` again. The recording
   stops.
3. `[87]` The title and top status show elapsed recording seconds. These indicators remain visible during screen learning.

## Session

1. The program saves the session automatically after some seconds.
2. Press `Ctrl+S`. The program saves the session immediately. `[22]` The message
   "Session saved" shows for five seconds.
3. Press `Ctrl+Q`. The program stops.

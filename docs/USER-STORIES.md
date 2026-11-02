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
- Screens: `F2` pads, `F3` chop, `F6` files, `F12` settings.
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

## Your own keys

1. `[64]` Create the file `~/.config/ptsr/keymap.toml`. Write in it only the keys
   that you want to change. The sections are the same as in `config/keymap.toml` of the
   project: `[global]`, `[edit]`, `[screens.pads]` and others.
2. `[64]` The line `"Ctrl+G" = "pad.mark_bank"` in `[screens.pads]` adds a key. Then
   `Ctrl+G` and `Ctrl+A` both mark the bank. `[76]` The old name `pad.scope_next` still
   works and does the same. `Ctrl+Shift+` and a letter is also a key: for example,
   `"Ctrl+Shift+Z" = "pad.mark_all"`. The letter alone still plays its pad.
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

## A project for a track

1. `[11]` Press `Ctrl+P`. The project list opens.
2. `[11]` Type a name. Press `Enter`. The program makes the project folder in
   `~/ptsr/projects` and opens the project. The project name is in the status line.
3. `[11]` Press `Ctrl+P` again. `I` `K` select a project. `Enter` opens it. Each project
   has its own pads, cuts and volumes.
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
   `Ctrl+Shift+A` puts a mark on all 128 pads. In Ghostty, first add the line
   `keybind = ctrl+shift+a=unbind` to the Ghostty config: Ghostty uses this key to select
   all text. Without the kitty keyboard protocol, `Ctrl+Shift+A` marks only the bank.
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
   each pad. Outside edit mode, `Backspace` still takes the sound off the selected pad.
10. Outside edit mode, `J` and `L` page the banks. `[43]` `PgUp` `PgDn` also page the
    banks in edit mode.
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
5. `[70]` The pad frame shows the group number: `①`…`⑧`. Knob 3 of the TONE page changes
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
   full. The title shows only the open page: `TONE 1/7`.

## Effects: drive, reverb, delay

1. `[56]` Each pad has its own drive. There are also two shared slots, FX 1 and FX 2.
   Each slot contains reverb or delay. A pad sends its own part of the sound to each slot.
   In a new project, FX 1 is reverb and FX 2 is delay.
2. Select a pad. Open edit mode (`Enter` or b10). Press `Tab` (or b4) until the FX page.
   Knob 1 changes drive. Knob 2 sets the send to FX 1. Knob 3 sets the send to FX 2. The
   send line shows the effect in the slot: `send 1  20 %  reverb`. `J` `L` change by
   1 %. Hold them for 10 %.
3. At 0 % the pad sounds as before. Drive adds dirt, and the volume almost does not
   change.
4. Press `Tab` again: the page FX1*. The star means a shared slot. The knob changes the
   sound of all pads that send to the slot. The `change` line shows `all pads`. Knob 1
   selects the type (reverb or delay). For reverb, knobs 2 and 3 are the room size and
   the tone. For delay, they are the time (10–2000 ms) and the repeats (up to 90 %). The
   echo always decreases to silence.
5. Press `Tab` again: FX2*. It is the same for the second slot. For two reverbs, put
   reverb in the two slots.
6. When you change the slot type, the old tail stops smoothly, without a click. If you
   set the old type again, its settings are the same as before.
7. The reverb tail and the echo continue after the pad stops.
8. In the panel on the right, the FX block is always visible. The block of a shared slot
   shows only while you are on its page.
9. On the pad frame: `↯` drive, `◌` send to reverb, `⋯` send to delay.
10. The project keeps all of these settings. An old project opens without effects.
11. Listen to `renders/56-drive.wav`, `56-reverb.wav`, `56-delay.wav`,
    `56-two-slots.wav`.

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

1. `[55]` Press `F2`. On the top frame of a pad, on the right, there are glyphs. They
   show the settings of the pad: `↕` pitch, `◣` envelope (attack, decay or release),
   `∿` filter (type, cutoff or resonance). `◻` means that the pad plays a part of the
   file: a slice or its own part.
2. A new pad with a full file has no glyphs. Set the pitch to 0 again: `↕` disappears.
3. After you chop, all pads with slices show `◻`. The pad with the full break does not.
4. Volume has no glyph. The mode line already shows it (`one-shot +3`).
5. Select a pad. In the panel on the right, the changed values are bright. The other
   values are dim. A block with a change has a dot: `─ FILTER ● ──`.
6. `F1` explains the glyphs in the section GLYPHS ON A PAD.

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

1. `[21]` Select a pad. Press button b10. Edit mode opens.
2. `[54]` Button b4 opens the next page, as `Tab` does. Button b3 opens the previous page.
   You do not have to select a parameter. Each parameter on the page has its own knob.
   `[54]` You can assign a knob for the pages in `F12`, group PADS · EDIT, row
   "page (knob)". The program divides its travel into sections, one for each page. The
   page changes immediately.
3. Knobs k1–k3 change the parameters of the page. `[58]` The parameter goes to the knob
   position immediately (`jump` mode, refer to "MIDI knobs"). Attack, decay and release
   follow a curve. Near zero, the knob is more accurate.
4. Press b10 again. The mode closes. The buttons and knobs go back to their usual
   functions.
5. On the chop screen, b10 also opens the edit of a slice. k1 and k2 move its bounds.
6. `[21]` On the chop screen, k3 moves the cursor. k4 and k5 set the start and the end of
   the region.
7. `[21]` Select a start pad. Press b9. The slices go into the plan in sequence from this
   pad. Press b9 again. The program assigns the slices. If the pads are not empty, b9
   confirms the replacement.

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
    ignores small rises in a sound. After `Ctrl+Y`, the marks are exactly on its cuts.
    The two use the same threshold.
    `[04d]` When the mode "from hit to hit" is on, `M` puts the cut on the nearest hit.
    The maximum distance is 90 ms from the point where you pressed. The status shows
    "marker snapped to hit". Set `N` to a different mode: the cut goes where you pressed.
8. Press `Y`. The program chops into equal parts: 2, 4, 8, 16, 32.
    `[04d]` Press `Ctrl+Y`. The program chops by hits. The first press puts cuts on the
    loudest hits. Each next press decreases the threshold: −12, −18, −24, −30 dB, then −6
    again. Quieter hits get their own cuts. The status names the threshold and the number
    of cuts. `Ctrl+Z` returns the cuts of the previous press.
    `[68]` `Ctrl+Y` replaces manual cuts, and the status tells you:
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

1. On the chop screen, press `P`. Assign mode starts from the first slice.
2. Hit the pads in sequence. Each hit puts the next slice in the list.
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

1. Open assign mode. Press `M`, `J`, `L`, `I`, `K` or `Ctrl+J`.
2. The cuts, the zoom and the cursor do not move. The assign plan does not change. The
   status line tells you: `Enter` applies, `Esc` closes.
3. Only these work: the pads, `Tab`, the slice selection, undo, `Enter` and `Esc`.

## Swap pads

1. Hit a pad. Press `Ctrl+X`. The pad goes "in hand". The panel shows
   `in hand: pad 3 · amen 3`. `[35]` The envelope moves with the pad.
2. Hit a different pad. The two pads change positions.
3. Press `Esc`. The pad goes back, and there is no swap.

## MIDI knobs

1. Press `F12`. Change the tab with `Tab`. Open the MIDI tab.
2. The list is folded into groups: "PADS · 8 actions · 2 learned". The group of the
   screen where you opened the settings is unfolded. The other groups are folded. `PgUp`
   and `PgDn` go from group to group. `→` unfolds a group. `←` folds it.
3. Select an action with `I` and `K`. Press the learn key. Turn a knob of the controller.
   `[74]` The knob is not bound yet. The row shows it in amber next to the old control:
   `none → k5`. If you touch a different control, it replaces the first. `Enter` binds
   the knob and saves it. `Esc` keeps all as it is. The config file does not change.
4. `[74]` Before `Enter`, the row shows what the control changes. If the control has a
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
8. `[59]` Connect the X-Session Pro for the first time. The program copies the functions
   from its profile into your bindings. For example, b10 opens edit mode, and k1–k3
   change the page. After that, all bindings are yours. There is no `(learned)` note. The
   row shows the control name: `b10 (note 70)`.
9. `[59]` Select a row. Press `Backspace`. The binding disappears fully. The profile does
   not return it, also after a restart.
10. `[59]` Below the row "knob response" is "reset to profile" with the device name.
    Press the learn key. The program asks. `Enter` returns the profile functions and
    removes your changes for this device. `Esc` keeps all as it is.
11. `[71]` Each bound knob and key does an action. At start, the program shows at the
    bottom each binding to an action that it does not have: file, key, name. The program
    removes such bindings from your config one time. Then it tells how many it removed.
    On the chop screen, knobs k6–k10 of the X-Session are now free. `Ctrl+K` is free on
    all screens.

## Bind the controller directly on the screen

1. `[47]` `[57]` On the pads, chop or files screen, press `F10`. The top line becomes
   magenta. It tells you in words what to do and where the binding goes: `PADS` or
   `PADS · EDIT · FILTER`.
2. `[52]` Select the item to bind with its own key. A pad key selects a pad (the pad does
   not sound). A command key selects a command (`M` on the chop screen selects "cut", and
   no cut occurs). `PgUp` `PgDn` select the banks. `I` `K` select the pad parameters
   (volume, pitch, attack, decay, filter, cutoff, resonance, start, end). The selected
   item has a highlight. Next to it is its control or `none`.
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
6. `[57]` If the control already has a binding on this layer, that item has the mark
   `✗ leaves`. The line gives a warning. `Enter` moves the control. If the control has a
   binding on a different layer, the line only shows `cc 28 on PADS: pad
   volume`. An example is the volume on the pads screen while you learn the cutoff in
   edit mode. The control stays on the other layer. In edit mode it changes the cutoff.
7. `[57]` A parameter needs a knob. A command needs a button. If the control is
   incorrect, the line explains:
   `it has a value to turn, a button only presses` (for a parameter),
   `it acts once per press, a knob would repeat it on every tick` (for a command).
8. `[57]` `Backspace` shows `cc 28 → none`. `Enter` removes the binding. `Esc` keeps it.
   `F10` closes the mode.
9. `[57]` The binding goes into the layer where you made it. If edit mode was open, the
   binding works in edit mode. If not, it works on the screen.

## Record sound

1. Hold `Ctrl+R`. The program records the microphone. Release the keys. The program stops
   and saves the recording. `[11]` The file goes to `samples` of the open project. If no
   project is open, it goes to the samples root from the settings. The recording is "in
   hand" immediately: hit a pad.
2. Press `Ctrl+T`. The program records its own sound. Press `Ctrl+T` again. The recording
   stops.

## Session

1. The program saves the session automatically after some seconds.
2. Press `Ctrl+S`. The program saves the session immediately. `[22]` The message
   "Session saved" shows for five seconds.
3. Press `Ctrl+Q`. The program stops.

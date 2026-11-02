# Live checks

This file lists the checks that the owner does by hand and by ear. Each section is one
OpenSpec change. The current behavior is in `openspec/specs/` and active changes. The steps
describe the current behavior, also when a later change replaced a part of the change.
The folder paths are relative to `openspec/`.

How to use this file:

1. Start the program with `task run`.
2. Do the steps of a section in sequence.
3. When a step is correct, put `x` in its brackets: `- [x]`.
4. When a step is not correct, write one line under the step. Write what you did and
   what you saw or heard.
5. Each problem goes into a new change. Change the spec first, then the code.

Notes:

- "(by ear)" marks a step that checks the sound. Use your headphones for these steps.
- Some steps use the WAV files in `renders/`. The command `task test` writes them.
- The pad keys are `1 2 3 4 / Q W E R / A S D F / Z X C V`. A pad number counts these
  keys in this order: pad 5 is `Q`, pad 7 is `E`. `2-Q` is pad `Q` of bank 2.
- "Superseded by NN" means that a later change replaced the behavior. Do the check of
  that later change.

## 11 — Projects: a folder with its samples

Folder: `changes/archive/2026-11-01-11-project-dir`

- [ ] Press `F12`. Open the row `Projects root`. Go into a folder with `→` and press
  `Enter`. Make sure that the row shows that folder. Do the same on the row
  `Samples root`. Restart the program. Make sure that both rows show your folders.
- [ ] Press `Ctrl+P`. Type `MyBeat` and press `Enter`. Make sure that the projects root
  has the folder `MyBeat` with `project.toml` and an empty `samples/`. Make sure that the
  window title and the status line show `MyBeat`.
- [ ] Put samples on pads. Chop one sample on `F3` and put the slices on pads. Restart the
  program. Make sure that `MyBeat` opens with the same pads and cuts.
- [ ] Open `project.toml` in a text editor. Make sure that a sample inside the project
  folder has a relative path. Make sure that a sample outside the folder has an absolute
  path.
- [ ] (by ear) Create a second project with other samples. Press `Ctrl+P` and open
  `MyBeat` again. Make sure that each project has only its own pads, banks, cuts and
  files folder. Make sure that each pad sounds at the pitch and volume that its panel
  shows. Listen to `renders/11-switch-nothing-leaks.wav`.
- [ ] Delete the folder of the open project in a file manager. Make sure that the program
  continues to operate. Make sure that the pads show "missing file" and the title shows no
  project name.

## 21 — Edit mode and the parameter standard

Folder: `changes/archive/2026-11-01-21-edit-mode`

- [ ] On the pads screen (`F2`), hit a pad. Press `Enter`. Make sure that the panel frame
  changes colour. Make sure that the frame title shows
  `EDIT · TONE ENV FILTER SOURCE FX FX1* FX2*` with TONE lit. Make sure that the rows
  above the help band do not move.
- [ ] Press `I` and `K` to select a parameter. Press `J` and `L` to change it. Make sure
  that a tap is a fine step and a held key is faster. Make sure that the knob numbers 1–3
  stand next to the parameters of the page.
- [ ] (by ear) Play a beat with the left hand while the edit is open. Make sure that the
  pads continue to play. Press `Enter`. Make sure that the edit closes.
- [ ] With the edit closed, press `H`. Make sure that the pad mode changes between
  `one-shot` and `gate`. Press `J` and `L`. Make sure that the bank changes and the
  selected pad stays at the same place.
- [ ] Press `F3`. Press `J` and `L` to move the cursor. Press `I` and `K` to zoom. Hold
  `Ctrl`, then hold `Alt`. Make sure that the start line, then the end line of the
  selected slice changes colour. Move each bound with `J` and `L` while you hold the key.
- [ ] Press `M` to cut. Press `,` and `.` to set the region. Press `Ctrl+,` to clear it.
  Press `Y` for equal parts. Press `O` and make sure that the green bar shows what `Space`
  plays. Press `P`, then `M`. Make sure that no cut occurs and the status line shows
  `assigning: Enter applies, Esc leaves`.

## 22 — Notices: the startup warning clears, the save speaks

Folder: `changes/archive/2026-11-01-22-notices`

- [ ] Start the program in a terminal without the kitty keyboard protocol, for example
  in tmux. Make sure that a warning about key releases shows.
- [ ] Press a key. Make sure that the warning goes away and the hint line comes back.
  Make sure that the status line shows `no key releases: gate pads play one-shot`.
- [ ] Press `Ctrl+S`. Make sure that `Session saved` shows. Make sure that it goes away
  after approximately 5 seconds.
- [ ] Change the volume of a pad. Do nothing for some time. Make sure that no message
  shows when the session saves automatically.

## 24 — The chop screen opens in silence

Folder: `changes/archive/2026-11-01-24-chop-hush`

- [ ] (by ear) Put a long sample on a one-shot pad. Hit the pad. Press `F3` while it
  plays. Make sure that the pad sound fades out at once, with no click.
- [ ] (by ear) On the chop screen, hit a pad that holds a slice of this sample. Make sure
  that the slice plays.
- [ ] (by ear) On the chop screen, hit a pad with a long sample. Press `F2` at once. Make
  sure that the pad continues to play.
- [ ] (by ear) Listen to `renders/24-chop-hush.wav`. Make sure that the fade has no click.

## 25 — The recording shows on the strip

Folder: `changes/archive/2026-11-01-25-record-strip`

- [ ] On `F2`, hold `Ctrl+R` and speak. Make sure that the strip above the pads shows
  your voice in the colour of the `● rec` mark. Release `Ctrl+R`. Make sure that the strip
  shows the live output again. Press `Esc` to drop the recording.
- [ ] Play music in a browser. Press `Ctrl+T`. Wait some seconds and press `Ctrl+T`
  again. Make sure that the strip shows the music during the recording.
- [ ] Select a pad and press `Enter`. Hold `Ctrl+R` and speak. Make sure that the strip
  shows the recording in place of the envelope. Release `Ctrl+R`. Make sure that the
  envelope comes back.

## 26 — The microphone is linked only when it is needed

Folder: `changes/archive/2026-11-01-26-mic-on-demand`

- [ ] Play music on your headphones. Start the program. Run `pw-top` in a different
  terminal. Make sure that the headphones drive the graph. Make sure that the microphone
  is not in the graph.
- [ ] Hold `Ctrl+R` and speak. Make sure that `pw-top` shows the microphone while you
  hold the key. Release the key. Make sure that the microphone leaves the graph.
- [ ] (by ear) Hit an empty pad to put the recording on it. Hit the pad again. Make sure
  that you hear your voice.
- [ ] Press `F12` and speak. Make sure that the input meter moves. Press `F2`. Make sure
  that the microphone leaves the graph.
- [ ] Use the program for some days. When the system sound breaks, write down the time.

## 27 — Every pad says what it plays: the whole file, a slice, or its own bounds

Folder: `changes/archive/2026-11-01-27-pad-source`

- [ ] Put a break on pad `1`. Press `F3` and cut the break into 8 slices with `Y`. Press
  `P`. Hit `Q W E R A S D F` in sequence. Press `Enter` to apply.
- [ ] On `F2`, hit pad `E`. Make sure that the panel shows `break.wav · slice 3`. Hit pad
  `1`. Make sure that the panel shows `break.wav · all`.
- [ ] Hit pad `E` and press `F3`. Make sure that the header shows `cuts of break.wav`.
  Make sure that slice 3 is selected and each slice shows its pad.
- [ ] On `F2`, hit pad `E` and press `Enter`. Press `Tab` until the SOURCE page. Press
  `K` to select `start`. Press `L`. Make sure that pad `E` shows `pad`. Make sure that the
  status shows `pad E no longer follows the cuts; Ctrl+Z links it back`.
- [ ] (by ear) Hit pads `W` and `R`. Make sure that they sound as before. Press
  `Ctrl+Z`. Make sure that pad `E` shows `slice 3` again.
- [ ] (by ear) Hit pad `1` and open its SOURCE page. Press `L` until the source is
  `slice 2`. Make sure that pad `1` plays slice 2.

## 30 — Files: a held arrow scrolls, the preview talks

Folder: `changes/archive/2026-11-01-30-files-list`

- [ ] Press `F6` in a folder with many files. Hold `↓`. Make sure that the selection
  continues to move while you hold the key.
- [ ] (by ear) Release the key on a file. Make sure that only that file sounds.
- [ ] (by ear) Restart the program and press `F6`. Make sure that the selected file
  sounds without `Space`. Press `Space`. Make sure that the sound stops and the help line
  shows `preview: off`.

## 31 — One layout for every screen

Folder: `changes/archive/2026-11-01-31-one-layout`

- [ ] Press `F2`, `F6`, `F3` and `F2`. Make sure that the wave strip and the pad grid
  stay at the same rows and columns. Make sure that only the right column and the strip
  content change.
- [ ] On `F6`, make sure that the folder path is directly above the first line of the
  file list.
- [ ] On `F3`, cut with `M`, zoom with `I` and `K`, and move the cursor with `J` and `L`.
  Make sure that each key operates as before.
- [ ] Make the terminal lower step by step. Make sure that the pad waves get fewer rows
  first. Then make sure that the strip gets lower, to a minimum of 3 rows. Make sure that
  the full grid always shows.
- [ ] Make the terminal approximately 60 rows high. Make sure that the strip is more than
  8 rows high. Make sure that it has the same height on `F2`, `F6` and `F3`.

## 32 — Hide the pads, the strip takes their rows

Folder: `changes/archive/2026-11-01-32-hide-pads`

- [ ] On `F2`, press `Ctrl+O`. Make sure that the grid goes away and the strip gets
  taller. Make sure that the pad line under the strip shows all banks and the selected
  pad, for example `pad W kick`.
- [ ] Hit pad `W`. Make sure that the pad line names pad `W` and flashes. Make sure that
  the bank row shows only pad `W` in white while it flashes.
- [ ] Press `F6`. Press `Enter` on a file, then hit a pad. Make sure that the file goes
  on that pad. Make sure that the path and the list use the full height of the right
  column.
- [ ] Press `F3`. Make sure that the wave is taller. Press `P`. Make sure that the grid
  comes back with the plan. Press `Esc`. Make sure that the grid goes away again.
- [ ] In a terminal with the kitty protocol, hold `Ctrl`. Make sure that the help line
  shows `pads: hidden` next to `O`.
- [ ] Restart the program. Make sure that the pads stay hidden. Press `Ctrl+O` and
  restart again. Make sure that the pads show.

## 33 — The preview shows where it plays

Folder: `changes/archive/2026-11-01-33-preview-playhead`

- [ ] (by ear) On `F6`, select a long file with the preview on. Make sure that a thin
  green line moves across the wave from left to right while the file plays.
- [ ] Select the next file. Make sure that the line starts again at the left edge.
- [ ] Press `Space`. Make sure that the line goes away. Press `Space` again and let a
  file play to its end. Make sure that the line goes away at the end.

## 35 — The UX pass: the musician's walk, fixed

Folder: `changes/archive/2026-11-01-35-ux-pass`

- [ ] (by ear) On `F3`, cut a sample into slices. Press `P`. Press `J` and `L`. Make sure
  that each press selects the previous or next slice and plays it. Hit a pad. Make sure
  that the selected slice goes on that pad and the cuts do not move.
- [ ] Put 9 slices on pads 1–9. Put the cursor at the start of slice 9 and press `,`.
  Make sure that pads 1–8 show `pad` and sound as before. Press `Ctrl+Z`. Press `Enter`.
  Make sure that the edit opens on the slice under the cursor.
- [ ] On `F3`, read the hint line in three states: no cuts, cuts with no slice on a pad,
  and slices on pads. Make sure that it tells the next step: `Y` or `Space` and `M`, then
  `P`, then how to move a bound.
- [ ] On `F6`, go to a folder with samples. Press `Ctrl+P` and create a project. Press
  `F6`. Make sure that the screen shows the folder from before. Press `Ctrl+P`. Make sure
  that the open project is marked and the help line shows the keys of the picker.
- [ ] Go to bank 2 and restart the program. Make sure that bank 2 is open. Hit a pad and
  press `Ctrl+X`. Make sure that the panel shows a short line, for example
  `in hand: pad 3 · amen 3`.
- [ ] On `F6`, press `Ctrl+B` on a folder. Go to its parent folder. Make sure that the
  folder shows one time, with `★`. In a terminal without the kitty protocol, open `F3`.
  Make sure that the help line shows `Ctrl+Z`, `Ctrl+Y`, `Ctrl+J L` and `Alt+J L`.

## 47 — Learn the controller on the screen itself

Folder: `changes/archive/2026-11-01-47-learn-on-screen`

Superseded by 52 and 57: the learn mode is now on `F10`, and the key of an element selects it. Do the check of 57.

## 48 — The pad panel in blocks, SOURCE last

Folder: `changes/archive/2026-11-01-48-panel-blocks`

- [ ] On `F2`, hit a pad and press `Enter`. Make sure that the panel shows the block
  lines TONE, ENV, FILTER, SOURCE and FX in this sequence, each above its rows. Make sure
  that the line of the open page is lit.
- [ ] Press `K` until `reso` on FILTER is selected. Press `K` again. Make sure that
  `source` on SOURCE is selected. Press `K` past the end of SOURCE. Make sure that `drive`
  on FX is selected.
- [ ] Press `Tab` again and again. Make sure that the lit page in the frame title moves
  through TONE, ENV, FILTER, SOURCE, FX, FX1* and FX2*.

## 49 — Loop the play on the chop screen

Folder: `changes/archive/2026-11-01-49-chop-loop`

- [ ] On `F3`, press `O` until the play mode is the slice. Press `Alt+O`. Make sure that
  the help line shows `loop: on`.
- [ ] (by ear) Put the cursor in slice 3 and press `Space`. Make sure that slice 3 plays
  again and again. Make sure that you hear no click at the seam.
- [ ] (by ear) While the loop plays, press `Alt+J` to move the end of slice 3 to the
  left. Make sure that the next turn ends at the new end. Make sure that the sound does
  not stop.
- [ ] (by ear) Press `Space`. Make sure that the sound stops. Press `Alt+O` and then
  `Space`. Make sure that the slice plays one time.
- [ ] Press `F2` and then `F3`. Make sure that the loop is off.
- [ ] (by ear) Listen to `renders/49-loop.wav`. Make sure that the seams have no click.

## 50 — The chop screen plays a slice as its pad does

Folder: `changes/archive/2026-11-01-50-play-as-pad`

- [ ] Put slice 3 of a break on pad `W`. On `F2`, set pad `W` to pitch −12 and low-pass
  at 500 Hz.
- [ ] (by ear) On `F3`, press `Alt+P`. Make sure that the help line shows `as pad`. Play
  slice 3. Make sure that it sounds one octave lower and darker.
- [ ] (by ear) While slice 3 plays, hit pad `W`. Make sure that the slice continues to
  play.
- [ ] (by ear) Put the cursor on a slice that is on no pad and play it. Make sure that it
  plays bare.
- [ ] (by ear) Press `Alt+P` again. Make sure that `as pad` goes away and slice 3 plays
  bare.
- [ ] (by ear) Listen to `renders/50-as-pad.wav`.

## 51 — Hits mode stops at strong hits, and the hits are marked

Folder: `changes/archive/2026-11-01-51-strong-hits`

Use the recordings in the folder `new`.

- [ ] On `F3`, open a recording. Press `N` until the help line shows `snap: hits`. Make
  sure that `▾` marks show above the wave only at the starts of strong hits.
- [ ] Press `L` again and again. Make sure that the cursor stops at the marks in
  sequence, at the start of the attack. Make sure that a small swell does not stop it.
- [ ] Press `Ctrl+Y`. Make sure that a mark shows at each new cut and at no other
  location in the region.
- [ ] Press `N` until the help line shows `snap: cuts`, then `snap: off`. Make sure that
  no marks show.

## 52 — Learn by the element's own key, save only after a yes

Folder: `changes/archive/2026-11-01-52-learn-by-keys`

Use the X-Session Pro. The "save" question is now the pending control and `Enter` (57).

- [ ] On `F2`, press `F10`. Make sure that the top line tells that the mode is on and
  what to do. Press pad key `Q`. Make sure that pad `Q` is lit and no sound plays.
- [ ] Press `H`. Make sure that "pad mode" is selected and no pad mode changes. On `F3`,
  press `F10` and `M`. Make sure that "cut" is selected and no cut occurs.
- [ ] Press `PgDn`. Make sure that the bank switch is selected. Press `I` and `K`. Make
  sure that they walk all pad parameters, the filter and the cutoff too.
- [ ] Select the cutoff. Turn a knob and press `Enter`. Press `F10` to leave. Turn the
  knob with the edit closed, then with the edit open on ENV. Make sure that the cutoff
  moves both times.
- [ ] Make sure that only elements with a control have a label.

## 53 — The learn mode moves on after a binding and asks only when it must

Folder: `changes/archive/2026-11-01-53-learn-flow`

Use the X-Session Pro. The bind without `Enter` and "always the screen's group" are
superseded by 57.

- [ ] On `F2`, press `F10` and pad key `1`. Hit a controller pad and press `Enter`. Make
  sure that the selection moves to pad 2. Hit the same controller pad again. Make sure
  that it does not show as pending for pad 2.
- [ ] (by ear) Bind controller pads to all 16 pads in this way. Press `F10` to leave.
  Make sure that each controller pad plays its pad.
- [ ] Press `F10` and `PgDn`. Turn a knob. Make sure that nothing is pending. Make sure
  that the top line tells that a button is necessary, and why.
- [ ] Select a parameter. Turn a knob and press `Enter` while you turn it. Make sure that
  the knob does not go to the next element.

## 55 — The pad shows what is set on it

Folder: `changes/archive/2026-11-01-55-pad-badges`

Superseded by 77: the pad frame has no glyphs; one `✱` on the mode line marks the settings. Do the check of 77.

- [ ] Put a sample on a fresh pad. Make sure that its top frame has no glyph.
- [ ] Open the edit of pad `R` with the whole file. Set the filter to low-pass and close
  the edit. Make sure that the frame of pad `R` shows `∿` and no other glyph.
- [ ] Set the filter to off and leave the cutoff away from its default. Make sure that
  the frame has no `∿`. Make sure that the FILTER line has no `●` and the cutoff is dim.
- [ ] Set the pitch to +2. Make sure that `⇕` shows. Set the pitch to 0. Make sure that
  `⇕` goes away.
- [ ] Make sure that the panel shows changed values bright and default values dim. Make
  sure that each block line with a change has `●`.
- [ ] Make sure that pads with slices show `◻` and a whole-file pad does not. Press `F1`.
  Make sure that it names each glyph.

## 56 — Drive on every pad, two shared effect slots fed by sends

Folder: `changes/archive/2026-11-01-56-pad-fx`

- [ ] (by ear) Listen to `renders/56-drive.wav`, `renders/56-reverb.wav`,
  `renders/56-delay.wav` and `renders/56-two-slots.wav`.
- [ ] (by ear) Open the edit of a pad. Press `Tab` until the FX page. Make sure that knob
  1 turns the drive, knob 2 send 1, knob 3 send 2. Turn the drive up. Make sure that the
  sound gets more grit at approximately the same loudness.
- [ ] (by ear) Set send 2 of pad `Q` to 20 % with a delay in FX 2. Hit pad `Q`. Make sure
  that you hear an echo after the pad stops. Make sure that the mode line of the pad ends with `✱`.
- [ ] (by ear) Press `Tab` until FX1*. Make sure that the block line shows
  `FX 1 * shared` and the change line shows `all pads`. Turn the size up. Make sure that
  each pad that sends to FX 1 has a longer tail.
- [ ] (by ear) On FX2*, turn knob 1 from delay to reverb. Make sure that the delay tail
  fades out with no click. Turn it back. Make sure that the delay has its old time and
  feedback.
- [ ] Press `Ctrl+S` and restart the program. Make sure that the drive, the sends and the
  slots are as before.

## 57 — The learn mode shows the new control next to the old one and waits for Enter

Folder: `changes/archive/2026-11-01-57-learn-confirm`

Use the X-Session Pro. If `F10` does nothing in your terminal, write the name of the
terminal.

- [ ] On `F2`, press `F10` and pad key `Q`. Make sure that the label of pad `Q` and the
  top line show its control, or `none`.
- [ ] Hit a different controller pad. Make sure that the new control shows next to the
  old one in the pending colour, for example `pad 5 → pad 6`. Press `Enter`. Make sure
  that the new control plays pad `Q` and pad `W` is selected.
- [ ] Press `I` or `K` to select the pitch. Turn a knob, then a different knob. Make sure
  that the row shows only the last knob as pending. Press `Esc`. Make sure that the old
  control stays.
- [ ] Turn a knob that already turns a different parameter. Make sure that the other row
  is lit in the "taken" colour and the top line warns you. Press `Esc`.
- [ ] Press `F10` to leave. Open the pad edit on FILTER and press `L` on the cutoff. In 5
  seconds, press `F10`. Make sure that the top line shows `cutoff · PADS · EDIT · FILTER`.
  Turn k5 and press `Enter`. Make sure that k5 turns the cutoff only while the edit is
  open.
- [ ] Select the cutoff and press `Backspace`. Make sure that `k5 → none` shows as
  pending. Press `Enter`. Press a button. Make sure that the top line tells that the
  cutoff needs a knob or a fader.

## 60 — Ctrl+X swaps the pads whole, with every setting

Folder: `changes/archive/2026-11-01-60-swap-whole`

- [ ] Set pad `Q` to a low-pass filter, pitch +3, a drive, a send and mute group 1. Keep
  pad `V` at its defaults.
- [ ] (by ear) Hit pad `Q`, press `Ctrl+X` and hit pad `V`. Make sure that pad `V` sounds
  as pad `Q` did, and pad `Q` sounds as pad `V` did.
- [ ] Hit pad `V`. Make sure that the panel shows the filter, the pitch, the drive, the
  send and the mute group of the old pad `Q`.
- [ ] Press `Ctrl+X` on a pad and hit an empty pad. Make sure that the sample moves and
  the first pad is empty.

## 61 — Ctrl+Z on the pads undoes the last change

Folder: `changes/archive/2026-11-01-61-pad-undo`

- [ ] Turn the volume knob of pad `Q` from 0 to −12 dB in one turn. Press `Ctrl+Z` one
  time. Make sure that pad `Q` is at 0 dB again.
- [ ] Press `Ctrl+A` until the scope is the bank. Raise the cutoff. Press `Ctrl+Z`. Make
  sure that all 16 pads get their cutoff back. Make sure that the status shows
  `undo: cutoff on 16 pads`.
- [ ] Swap pads `Q` and `V` with `Ctrl+X` and press `Ctrl+Z`. Clear pad `Q` with
  `Backspace` and press `Ctrl+Z`. Make sure that each step comes back.
- [ ] Set FX 2 from delay to reverb and press `Ctrl+Z`. Make sure that FX 2 has the delay
  with its time and feedback.
- [ ] Raise the volume of pad `Q`. Press `Ctrl+A` three times, change the bank and hit
  pad `W`. Press `Ctrl+Z`. Make sure that only the volume of pad `Q` comes back.
- [ ] Restart the program. Press `Ctrl+Z` on `F2`. Make sure that the status shows
  `nothing to undo`.

## 64 — The owner's keymap in the user config, over the shipped one

Folder: `changes/archive/2026-11-01-64-user-keymap`

- [ ] Write `~/.config/ptsr/keymap.toml` with the line `"Ctrl+G" = "pad.scope_next"`
  under `[screens.pads]`. Start the program from your home folder. Make sure that both
  `Ctrl+G` and `Ctrl+A` change the scope.
- [ ] Add `"Ctrl+A" = "none"` under `[screens.pads]` and restart. Make sure that `Ctrl+A`
  does nothing on the pads screen.
- [ ] Add `Q = "pad.scope_next"` and restart. Make sure that `Q` plays its pad. Make sure
  that the status line tells that a pad key cannot be a command.
- [ ] Add a line that is not correct TOML and restart. Make sure that the status line
  names the file and the line. Make sure that the shipped keys operate.
- [ ] Delete your test lines from the file.

## 66 — The files screen: a fresh list, a visible filter, the name over the wave

Folder: `changes/archive/2026-11-01-66-files-fresh`

- [ ] Open a folder on `F6`. Press `F2`. Save a new file into that folder from a
  different program. Press `F6`. Make sure that the new file is in the list and the
  selection did not move.
- [ ] Type `/sn` and press `Enter`. Make sure that the list shows only files with `sn`.
  Make sure that the line above the list shows `filter: sn · Esc clears`.
- [ ] Press `Esc`. Make sure that the full folder shows and the filter line goes away.
- [ ] Select a file. Make sure that the wave label shows the file name and its length,
  for example `kick.wav · 0.33 s`.

## 67 — Enter lays the slices in order from the selected pad, with no pad hit

Folder: `changes/archive/2026-11-01-67-assign-enter`

- [ ] On `F2`, go to bank 2 and hit pad `Q`. Open a chopped sample on `F3`.
- [ ] Press `P`, then `Tab` for "in order". Press `Enter`. Make sure that the plan puts
  slice 1 on pad `Q` of bank 2 and the next slices after it. Make sure that the summary
  shows.
- [ ] Press `Enter` again. If pads are taken, make sure that the overwrite question shows
  first. Make sure that the slices go on the pads.
- [ ] (by ear) Hit the pads. Make sure that each pad plays its slice.

## 68 — State you can see

Folder: `changes/archive/2026-11-01-68-state-seen`

- [ ] On `F2` in bank 2, press `Ctrl+A` until the scope is the bank. Make sure that the
  help line shows `mark: off · scope: bank 2`. Set the scope back to the pad. Press `M`,
  hit three pads. Make sure that it shows `mark: on, hit pads · scope: 3 marked`.
- [ ] On `F3`, press `Y` until 16 slices. Press `Ctrl+Y`. Make sure that the status shows
  `by hits at -6 dB: N slices (were 16) · Ctrl+Z brings them back`. Press `Ctrl+Z`. Make
  sure that the 16 slices come back.
- [ ] On `F6`, go into a folder, select a file and press `Ctrl+B`. Make sure that the
  status names the folder with `(the folder of this file)`. Press `Ctrl+B` again. Make
  sure that the status shows that the folder is no longer a favorite.
- [ ] On `F3`, press `Space`. Make sure that the header shows `LISTEN ▶`. Press `Space`
  again. Make sure that the header shows `LISTEN`.
- [ ] In a terminal without the kitty protocol, make sure that the status shows
  `no key releases: gate pads play one-shot`. Press `F1`. Make sure that a row names
  Ghostty, kitty, WezTerm and foot.

## 69 — F3 on a slice pad opens on that slice

Folder: `changes/archive/2026-11-01-69-f3-on-slice`

- [ ] Put 8 slices of a break on pads. Hit the pad with slice 5 and press `F3`. Make sure
  that slice 5 is selected in the middle of the view, with half its length on each side.
  Press `K` until the full file shows.
- [ ] On `F2`, open the edit of the pad with slice 3. On SOURCE, move its start a little.
  Press `Esc` and `F3`. Make sure that the view is on the piece of the pad. Make sure
  that slice 3 is selected and the cuts did not change.
- [ ] Hit a pad that plays the whole break and press `F3`. Make sure that the break opens
  with no cuts. Make sure that the status tells that the older chop stays with its pads.
  Press `Ctrl+Z`. Make sure that the older chop comes back.

## 70 — Mute groups

Folder: `changes/archive/2026-11-01-70-mute-groups`

- [ ] Put an open hat on pad `Q` and a closed hat on pad `W`. Press `M`, hit `Q` and
  `W`, press `M`. Press `Enter` and select `mute` on TONE. Press `L` one time. Make sure
  that the panel shows `mute    1` and the line under both names says `mute 1`.
- [ ] (by ear) Hit `Q`, then `W`. Make sure that the open hat stops at once with no click.
  Make sure that the closed hat plays fully.
- [ ] (by ear) Put pad `E` in group 2 and keep pad `R` off. Hit `Q`, `E`, `R`, then `W`.
  Make sure that only `Q` stops.
- [ ] (by ear) Put pads 1-1 and 2-1 in group 3. Hit 1-1, press `PgDn` and hit 2-1. Make
  sure that 1-1 stops. Listen to `renders/70-mute-groups.wav`.
- [ ] Press `Ctrl+Z` and make sure that the last group change goes back. Restart the
  program. Make sure that the groups stay. Press `F1`. Make sure that it has a row for
  `mute N`.
- [ ] Close the edit. Make sure that the first panel row ends with `Enter: edit`. Press
  `Tab`, `Shift+Tab`, `I` and `K`. Make sure that they walk the pages and parameters. Make
  the window 100×30. Make sure that closed blocks fold with `▸` and `▲ N more` / `▼ N more`
  show.

## 74 — Learn on F12 shows what it changes before it saves

Folder: `changes/archive/2026-11-01-74-learn-preview`

- [ ] Connect the X-Session Pro. Press `F12` and open the MIDI tab. Select "bank next" in
  GLOBAL, press `Enter` and press b9. Make sure that the row shows b9 in amber next to the
  old control. Press `Enter`. Make sure that b9 goes to the next bank.
- [ ] Select "pad mode" in PADS, press `Enter` and press b9. Make sure that a grey line
  under the row shows `b9 on GLOBAL: bank next · stays`.
- [ ] Press b8, then b9 again. Make sure that each new button replaces the one in amber.
  Press `Enter`. On the pads screen, make sure that b9 changes the pad mode. On the files
  screen, make sure that b9 goes to the next bank.
- [ ] On the MIDI tab, select "marking on / off" in PADS, press `Enter` and press b9. Make
  sure that the "pad mode" row becomes magenta with `✗ loses b9`. Press `Esc`. Make sure
  that the rows show the same controls as before. On the pads screen, make sure that b9
  still changes the pad mode.
- [ ] Do the step above again, but press `Enter`. Restart the program. Make sure that b9
  toggles the marking on the pads screen and the "pad mode" row has no b9.
- [ ] Make sure that b9 goes to the next bank. On the MIDI tab, select a row in another
  group, press `Enter` and press b9. Make sure that the selection stays on that row and
  the bank does not change before you press `Enter` again. Press `Esc`.

## 12 — The program is named ptsr

Folder: `changes/archive/2026-11-01-12-rename-ptsr`

- [ ] Make sure that `~/.config/ptsr/` does not exist and `~/.config/arasaka-404/` does.
  Start the program with `task run`. Make sure that the status line says
  `settings moved from ~/.config/arasaka-404`.
- [ ] Make sure that the pads have the same sounds as before, and the settings screen
  (`F12`) shows the same audio device and the same MIDI bindings.
- [ ] Make sure that your own keys from `keymap.toml` work as before. Make sure that
  `~/.config/arasaka-404/` and `~/.local/share/arasaka-404/` are still there.
- [ ] Restart the program. Make sure that the status line does not say that the settings
  moved.
- [ ] If you linked the sound to `arasaka-404` by hand (for example in a patchbay), link it
  again to the `ptsr` client.

## 76 — Marks are the scope, and a pad can be reset

Folder: `changes/archive/2026-11-02-76-pad-marks`

- [x] On `F2`, open bank 2 (`Ctrl+2`) and press `Ctrl+A`. Make sure that all 16 pads of
  the bank have a dark yellow fill and a yellow frame, and that the top line shows the
  yellow plate `MARK · hit pads · 16 marked`.
- [x] Hit two pads. Make sure that they lose the yellow. Open bank 5 (`Ctrl+5`) and hit
  three pads. Make sure that they become yellow, and that the bank row shows yellow dots
  at bank 2 and bank 5 where the marks are.
- [x] Press `Enter` and turn the volume down. Make sure that every marked pad in both
  banks is quieter (by ear).
- [x] Still in the edit, press `Backspace`. Make sure that a question asks
  `Reset 17 pads to clean?`. Press `Enter`. Make sure that the pads sound as freshly
  loaded, with the same cuts (by ear). Press `Ctrl+Z`. Make sure that the settings come
  back (by ear).
- [x] Press `Esc`. Make sure that the edit closes and the marks stay, and that the plate
  says `17 MARKED · Esc drops`. Press `Backspace`. Make sure that a question asks
  `Clear 17 pads?`. Press `Esc`. Make sure that the sounds stay.
- [x] Press `Esc`. Make sure that the marks and the plate are gone. Press `Backspace`.
  Make sure that only the selected pad loses its sound, and that nothing asks.
- [ ] Listen to `renders/76-reset-sounding.wav`. Make sure that the quiet, dull tone becomes
  loud and open in the middle, without a click (by ear).

## 77 — A quiet pad grid

Folder: `changes/archive/2026-11-02-77-pad-glyphs`

- [x] Look at the pad grid. Make sure that no pad frame carries glyphs.
- [x] Load a sample on a fresh pad. Make sure that the line under its name is `⇢`.
- [x] Put the pad into mute group 2 and lower it by 6 dB. Make sure that the line is
  `⇢ -6 mute 2`, with no `✱`.
- [x] Raise its pitch. Make sure that the line is `⇢ -6 mute 2 ✱`. Turn the pitch back to
  0. Make sure that the `✱` goes.
- [x] Press `H`. Make sure that the line is `⇥ -6 mute 2`, and that `⇢` and `⇥` are easy
  to tell apart.
- [x] Press `F1`. Make sure that it names `⇢` one-shot, `⇥` gate, `mute N` and `✱`.

## 79 — Project sounds stay separate

Folder: `changes/79-project-sound-isolation`

- [ ] Create project A. Load a long sample on a pad and slices on other pads. Change a
  pad's volume or pitch. Note the sounds and settings.
- [ ] (by ear) Start the long sample. While it sounds, use `Ctrl+P` to create an empty
  project B. Make sure that the old sound fades out without a click and stays silent.
- [ ] (by ear) Hit the same pads in B on the keyboard and on the MIDI controller. Make
  sure that they are silent. Hit pads while B opens. Make sure that the hits are silent
  and do not sound later.
- [ ] (by ear) Use `Ctrl+P` to return to A. Hit its pads on the keyboard and on the MIDI
  controller. Make sure that the samples and slices play with their saved settings.
- [ ] (by ear) Listen to `renders/79-project-switch.wav`. Check that the old sound fades
  without a click and that no old sound returns after the switch.

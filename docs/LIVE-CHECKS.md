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

- Section 84a is offline. Do not start the sampler for that section; use its benchmark command.
- Section 87 separates its offline catalogue from owner-only device checks. Do not start the sampler to view the catalogue.
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
  `EDIT · TONE ENV FILTER SOURCE FX` with TONE lit. Make sure that the rows
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
  through TONE, ENV, FILTER, SOURCE and FX.

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

- [ ] On `F2`, press `F10`. Check the learning block for the mode and next step.
  From 87, this block uses the bottom three rows. The top status stays visible.
  Press pad key `Q`. Check that pad `Q` is lit without a new sound.
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
  that the learning block explains why a button is necessary.
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

## 82 — Compact shared FX screen

Folder: `changes/archive/2026-11-03-82-shared-fx-screen`

Agent verification on 2026-11-02: the full automated suite passed. Recorded input
walkthroughs checked file loading and filtering, rename, marks, reset/clear confirmation,
swap, undo, chopping and assignment, project save/reopen, and F4 transitions from each
screen. Tests also checked MIDI release across screens, banks and channels. The live
100x32 screen showed compact cards. The isolated process reported `JACK unavailable`.
A separate owner process already had a PipeWire node named ptsr, so the agent stopped
its test session. No live sound or physical controller behavior was accepted from this run.
Small and large layouts were checked with TestBackend, not an interactive terminal.
Owner acceptance on 2026-11-03: the owner first confirmed that the replacement
X-Session profile works, then accepted all checks of change 82 and authorized closure.
The checks below are marked from that owner acceptance, not from an agent live run.

The X-Session replacement belongs to change 82. The physical control table is in
`docs/USER-STORIES.md`, section "Set up a pad with the X-Session Pro".
The profile applies automatically when the port name contains `X-Session`.
There is no manual profile selector. An existing user map keeps its old assignments until
reset, including old Play roles and destructive delete, commit or replacement assignments.

- [x] Stop the running program. Start the newly built program. Connect the X-Session Pro.
  Before reset, check that the existing map still has its previous assignments.
- [x] Press `F12`. Open MIDI. Select `reset to profile` for `X-Session Pro`.
  This selects a device to reset, not a profile. Press `Enter` to open the question.
  Press `Esc`. Check that the existing bindings remain unchanged, including after a restart.
- [x] Select that reset row again. Press `Enter` to open the question.
  Press `Enter` again to confirm. Check that the new physical defaults replace this device's map.
  This deletes custom bindings only for this device. Check that other devices keep their bindings.
  Restart the program. Check that the replacement map remains.
- [x] Press left Headphone (b2, note 44). Check that Files opens.
  Press right Headphone (b3, note 45). Check that FX opens.
- [x] Check left backward (b4, note 46) and left forward (b1, note 43).
  They select previous / next pads on Pads, files on Files, slices on Chop and blocks on FX.
  In edit, they select previous / next pages. A release must not repeat the command.
- [x] Press right backward (b5, note 56) and right forward (b6, note 57).
  Check previous / next bank. Check that button releases do not change banks again.
- [x] On Pads, press left Cue (b7, note 58). Check that pad edit opens.
  Turn left numbered knobs 1–3 (k1–k3, CC24–26). Check the current page parameters.
  Close edit with left Cue. Open it again and close it with right Cue (b8, note 59).
- [x] On Files, press left Cue to hold a file. Press it again. No file is placed.
  Press right Cue to open Chop.
- [x] Start a layout plan with the keyboard. Press right Cue. The plan must not apply.
  Open a replacement question with the keyboard. Neither Cue must confirm it.
  On Chop, press left Cue to cut. Press right Cue to open slice edit.
  Check that either Cue closes edit without another cut.
  Outside Files, Chop and edit, check that right Cue goes back.
- [x] On Files, test left Play (b10, note 70) and right Play (b9, note 69) separately.
  Each must toggle file preview. On Chop, each must start and stop preview.
  Repeat with edit open. Neither Play must open or close edit, delete cuts or apply a layout.
  Check that both Plays do nothing on Pads and FX. Releases must not repeat commands.
- [x] On Chop with edit closed, turn k1–k5 (CC24–28).
  Check slice start, slice end, cursor, region start and region end respectively.
  Check right Mid (k11, CC35) for threshold and right Low (k12, CC36) for zoom.
- [x] On each screen and in edit, move left Volume (fader_b, CC11), left Pitch (fader_a, CC12)
  and right Volume (pitch_a, CC14). Check selected pad volume, selected pad pitch and master volume respectively.
  Check that right Pitch (pitch_b, CC15) and crossfader messages CC17 / CC20 change nothing.
- [x] On a disposable project, check every default button on each screen and in edit.
  No default may delete a cut, apply a layout or confirm replacement.
  Use keyboard commands for layout, or explicitly learn a commit control.
  An old destructive assignment remains active until reset. Do not use the old map for this safety check.

- [x] Open `F4`. Check two compact cards with type and two parameter rows. In a wide
  terminal they are beside each other. In a narrow terminal they are stacked. Cards do
  not grow to fill empty space. The selected block and parameter are clear.
- [x] Press `Tab` and `Shift+Tab`. Only the selected block changes. `I` and `K` select a
  parameter. `J` and `L` change it. Check `F1` and the bottom help.
- [x] Play keyboard and MIDI pads while changing effects. Hold a gate pad, change the
  screen and bank, then release it. The original pad must stop.
- [x] Set the sends on two pads to FX 1. On `F4`, change size and tone. Both pads must
  use the changed effect. Select FX 2 and use the same three knobs. FX 1 must stay unchanged.
- [x] In pickup mode, change FX 1, select FX 2 and move the same knob. FX 2 must wait
  for its own value. Check a second device and the scale and jump modes too.
- [x] Check the old page knobs. Assign a replacement with `F10`. The old knob must stop
  controlling that action. Clear the new assignment, restart, and check that neither returns.
- [x] Press `Ctrl+Z` after a keyboard or MIDI effect change. Check the previous setting.
  Save with `Ctrl+S` and restart. Check both slots, drive and sends.
- [x] Return to `F2`. Check only TONE, ENV, FILTER, SOURCE and FX. The previous pad
  parameter stays selected. Shared slot settings must not appear in the pad panel.
The profile update changes no sound algorithms. It needs no new sound render.
Use the existing change 82 WAV for the listening check.

- [x] (by ear) Listen to `renders/82-shared-fx-screen.wav`. Check reverb and delay,
  parameter changes and undo for clicks. Compare with `56-reverb.wav`, `56-delay.wav`
  and `56-two-slots.wav`.

## 56 — Drive on every pad, two shared effect slots fed by sends

Folder: `changes/archive/2026-11-01-56-pad-fx`

- [ ] (by ear) Listen to `renders/56-drive.wav`, `renders/56-reverb.wav`,
  `renders/56-delay.wav` and `renders/56-two-slots.wav`.
- [ ] (by ear) Open the edit of a pad. Press `Tab` until the FX page. Make sure that knob
  1 turns the drive, knob 2 send 1, knob 3 send 2. Turn the drive up. Make sure that the
  sound gets more grit at approximately the same loudness.
- [ ] (by ear) Set send 2 of pad `Q` to 20 % with a delay in FX 2. Hit pad `Q`. Make sure
  that you hear an echo after the pad stops. Make sure that the mode line of the pad ends with `✱`.
- [ ] (by ear) Press `F4`. Make sure that both shared blocks are visible and FX 1 is
  selected. Turn the size up. Make sure that each pad that sends to FX 1 has a longer tail.
- [ ] (by ear) Press `Tab` to select FX 2. Turn knob 1 from delay to reverb. Make sure that the delay tail
  fades out with no click. Turn it back. Make sure that the delay has its old time and
  feedback.
- [ ] Press `Ctrl+S` and restart the program. Make sure that the drive, the sends and the
  slots are as before.

## 57 — The learn mode shows the new control next to the old one and waits for Enter

Folder: `changes/archive/2026-11-01-57-learn-confirm`

Use the X-Session Pro. If `F10` does nothing in your terminal, write the name of the
terminal.

- [ ] On `F2`, press `F10` and pad key `Q`. Check its label and the learning block for its control or `none`.
  From 87, the learning block uses the bottom three rows. The top status stays visible.
- [ ] Hit a different controller pad. Check the pending control beside the old control, such as `pad 5 → pad 6`.
  Press `Enter`. Check that the new control plays pad `Q` and selects pad `W`.
- [ ] Select pitch with `I` or `K`. Turn a knob. Turn a different knob.
  Check that only the last knob is pending. Press `Esc`. Check that the old control remains.
- [ ] Turn a knob assigned elsewhere on the same layer. Check the losing row and the warning in the learning block.
  Press `Esc`. Check that no assignment changed.
- [ ] Press `F10`. Open pad edit on FILTER. Change cutoff with `L`.
  Press `F10` within five seconds. Check `cutoff · PADS · EDIT · FILTER` in the learning context.
  Turn k5. Press `Enter`. Check that k5 changes cutoff only inside edit.
- [ ] Select cutoff. Press `Backspace`. Check the pending `k5 → none` detail.
  Press `Enter`. Press a button. Check the explanation that cutoff needs a knob or fader.

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

These steps create custom bindings. They do not describe the replacement defaults.
Reset the X-Session map as described in section 82 when you finish these steps.

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
Current update: `changes/archive/2026-11-04-86a-remove-legacy-migration`

The owner accepted 86a on 2026-11-04 and authorized archive and merge of PR #5.
This is overall acceptance. Individual keyboard, controller and listening results remain unreported.
The manual boxes below stay open.

Startup no longer copies older program folders. Keep all owner folders on disk.
The historical archive stays unchanged. The checks below replace its obsolete migration checks.

**Keyboard**

- [ ] Start with `task run`. Open `F12`. Check the saved sound device and MIDI bindings.
- [ ] Open `F2`. Check the saved pads and an owner key from `keymap.toml`.
- [ ] Restart. Check that the same settings and session load, without a settings-moved notice.

**Controller**

- [ ] Use an existing learned control. Check that its saved action still works.

**Listening**

- [ ] On `F2`, play a previously saved pad. Check that its sound stays the same.

No new render: this change removes startup file copying, not sound processing.
Automated file-loading tests do not replace these manual checks.

**Local automated results (86a)**

- `task check`: passed. 945 workspace tests, including 669 App tests. Format, Clippy and benchmark checks passed.
- `task build`: passed. Release build completed.
- `openspec validate --all --strict --no-interactive`: `Totals: 16 passed, 0 failed (16 items)`.
- Main-spec coverage passed. The prose cleanup preserved all 679 scenario texts. This separate behavior change replaces three obsolete migration scenarios with two file-loading scenarios, for 678 total.
- The new tests call the production settings, keymap and session loaders. They cover conflicting legacy files, current defaults, saved keys, MIDI bindings, pad state and unchanged source files. These tests do not start the sampler.

All benchmark maximum summaries were within target in this run. Earlier maximum-budget and slow-core CPU issues remain open.

**Archive verification (2026-11-04)**

The archive keeps all five change artifacts. Both added requirements match the main spec, and both removed requirements are absent.
Archive used `--skip-specs` because the delta was already fully applied. The main project spec stayed byte-identical.
After archive, `task check` passed all 945 tests, including coverage of 678 main-spec scenarios. The release build passed.
Strict validation reported `Totals: 15 passed, 0 failed (15 items)`. There are no active changes.
All benchmark maximum summaries were within target in this run. Earlier timing issues remain open.
Independent review of `59d437a` found no lost limits or changed behavior in the prose cleanup.

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

Folder: `changes/archive/2026-11-02-79-project-sound-isolation`

- [x] Create project A. Load a long sample on a pad and slices on other pads. Change a
  pad's volume or pitch. Note the sounds and settings.
- [x] (by ear) Start the long sample. While it sounds, use `Ctrl+P` to create an empty
  project B. Make sure that the old sound fades out without a click and stays silent.
- [x] (by ear) Hit the same pads in B on the keyboard and on the MIDI controller. Make
  sure that they are silent. Hit pads while B opens. Make sure that the hits are silent
  and do not sound later.
- [x] (by ear) Use `Ctrl+P` to return to A. Hit its pads on the keyboard and on the MIDI
  controller. Make sure that the samples and slices play with their saved settings.
- [x] (by ear) Listen to `renders/79-project-switch.wav`. Check that the old sound fades
  without a click and that no old sound returns after the switch.

## 80 — Name a pad

Folder: `changes/archive/2026-11-02-80-rename-pad`

The owner checked the rename and cursor (2026-11-02).

- [x] Put a sample on a pad. Press `Ctrl+N`. Make sure that a cursor shows after the
  name. Clear the line and make sure that the cursor stays visible in the empty pad.
  Type a long name with wide characters. Make sure that its end and the cursor stay
  inside the frame. Press `Enter` and make sure that the cursor goes away.

## 81 — Drive sound after optimization

Folder: `changes/archive/2026-11-02-81-drive-performance`

The owner accepts the result for now; the sound can be reviewed again during use.

- [ ] (by ear) Listen to `renders/81-drive-comparison.wav`. The first part is the old
  drive, the second is the optimized drive. Make sure they sound the same, including the
  change in drive. There is a short silence between them.

## 83a — Keymap cleanup

Folder: `changes/archive/2026-11-03-83a-keymap-cleanup` (`plan/controller-first-patterns`)

Owner acceptance on 2026-11-03: the owner chose to accept this change without
completing the manual checks. Open boxes below mean **not verified by the owner**;
do not read them as failed or completed. Bugs found later get a separate change.

Agent check on 2026-11-03: `task check` and `task build` passed. In an isolated
120x40 tmux terminal, the app reported `JACK unavailable`, with no `ptsr` node in
the owner's PipeWire graph. The agent saw U/O change banks, loaded a sample,
used Y and H to cut it, selected hit slices with pad keys, stepped H back with
Shift+H, changed assign order with P and Tab, copied a pad with Ctrl+C and
undid the copy with Ctrl+Z. Old Ctrl+R, Ctrl+Y and Ctrl+N showed moved-key
notices. Alt+R reported `No sound: there is nothing to record from.` The agent
did not hear the sound, test microphone input, verify a physical controller or
check owner overrides. The real SMC-PAD profile is work for change 91.

The keyboard checks below remain available for a later pass; the controller
pass cannot be inferred from automated MIDI tests.

- [ ] On `F2`, hold `Alt+R` and speak. Make sure that the strip shows your voice. Release
  it. Press `Alt+R` again: the recording stops only on the second press in a terminal
  without releases.
- [ ] On `F2`, press `U`, then `O`. Make sure that bank 2 opens and bank 1 comes back,
  with the same pad selected. Press `L`: make sure that a value changes, not the bank.
  Press `Enter` (edit open) and repeat `U` / `O` / `L`: the same behavior.
- [ ] On `F2`, press `Ctrl+C`. Make sure that the flash says `COPY · hit a pad` and that
  the program does not quit. Hit an empty pad: make sure that the copy lands with the
  numbered name from section 87a and that the hit plays nothing. Press `Ctrl+Z`: make sure that the
  target comes back. Press `Ctrl+Q`: make sure that the program quits.
- [ ] On `F3`, press `H`. Make sure that the status says `by hits at -6 dB: N slices`.
  Press `Shift+H`: make sure that the threshold steps back up one list value. Press it
  at `-6 dB`: make sure that the status says `by hits stays at -6 dB` and that no undo
  step was added.
- [ ] On `F3`, press `Ctrl+G`. Make sure that the pad the chop came from normalizes to a
  `-1 dB` peak. Press `P` and `Tab`: make sure that the plan switches `in order` /
  `by hand` and that `Tab` outside a plan does nothing.
- [ ] On `F2` and `F3`, press `Ctrl+R`, `Ctrl+Y` and `Ctrl+.` one by one. Make sure that
  no action runs; only the moved-key flash may name the new key.
- [ ] In your `keymap.toml`, bind `H` on the chop screen to another action. Start the
  program. Make sure that a startup warning names `H on chop`, that your file is
  unchanged, and that a key press dismisses one warning at a time.
- [ ] (by ear) Listen to `renders/83a-pad-copy.wav`. The first voice is the source, the
  second is the copy placed on another pad: make sure that they sound the same.

## 84 — Keyboard and controller pad parity

Folder: `changes/archive/2026-11-03-84-input-parity` (`plan/controller-first-patterns`)

Owner acceptance on 2026-11-03: the owner reported that they checked and accepted
change 84, then explicitly authorized its archive and a local signed-off commit.
This is overall acceptance, not a separate result for every check below or the WAV.
The boxes stay open where no individual result was reported; they do not mean failure
or rejection of the change. No controller is available, so its physical pass remains
unverified. Automated raw-MIDI tests do not replace that pass. No agent live, listening
or physical-controller check is implied. The earlier missing Chop pad playback line
was repaired in 2.3; F2 PLAY followed in 2.4.

### Prepare both passes

Use a disposable project. These checks place, copy, swap and replace pad sounds.
Start each pass with the same samples and banks. Use a fresh chop for the first check,
with no slices linked to pads. Keep an empty target and an occupied target in bank 2.
For gate checks, use a linked slice at least 10 seconds long: cut a sample of at least
20 seconds into two slices and assign one to `2-W`. On Pads, set it to gate with `H`
and its release to 20 ms on ENV. Keep both FX sends at 0, so an effect tail cannot hide
the stop. Use a terminal with the kitty keyboard protocol for keyboard releases.

For the controller pass, use notes 36–51 from a device without a profile, or controls
explicitly bound to `pad.1`…`pad.16` in the GLOBAL group on `F12` MIDI. Record the device
and preset below. The X-Session Pro has no physical pad grid; do not treat its default
command buttons as pad hits. Its existing safe command map stays unchanged. The SMC-PAD
profile and hardware measurements remain work for 91. A learned button that sends only
127 can test full hits, but not low/high velocity. Record that limit instead of a pass.
Use the keyboard for screen, bank, plan and confirmation commands without bindings.

### Keyboard pass

- [ ] On F2, hit the long pad. PLAY must change from dim to blue with `▶`, without
  moving following title text. Change bank, open pad edit with `Enter`, hide the grid
  with `Ctrl+O`, then return to F2 from F4: PLAY stays lit while any pad still plays.
  With two pads sounding, stop one: it stays lit until the last voice ends, including
  gate release. Let an effect tail continue after that: PLAY is dim. Check with MARK
  present too: its text and yellow color stay, and its column does not move on start/stop.
- [ ] (by ear) On a fresh Chop, use `Y` until there are 32 slices. Press `Ctrl+1` and
  hit `3`, then press `Ctrl+2` and hit `3`. Check that slice 3, then slice 19 is selected
  and previewed once. No pad layout is applied.
- [ ] On that Chop, press `Ctrl+1`, hit `1` to select slice 1, then press `P` and hit `Q`.
  Press `Ctrl+2` and hit `W`. Check the plan: slice 1 on `1-Q`, slice 2 on `2-W`.
  Each planned slice previews once; no pad changes yet. Press `Esc`: no layout is
  applied. Repeat the plan and use `Enter` to apply, with confirmation if asked.
- [ ] (by ear) On Pads, change a linked pad's pitch to +12 and its filter to low-pass
  at about 1200 Hz. Hit it, then open its slice with `F3` and hit it again. Check the
  same higher, darker pad sound and selected slice, without a second preview sound.
- [ ] On Chop, hit a linked pad without Space. Check the thin green line moving through
  its slice while it sounds. Hit it again: the line starts again. Open slice edit with
  `Enter` and repeat. The owner's cursor does not follow playback; Space and M keep
  their previous behavior. Let the sound end or release a gate: the line disappears.
  Check once with pitch +12 and in each wave mode available in your terminal. A line
  outside a zoomed view is hidden, not pinned to its edge. If Space preview also runs,
  its existing line has priority until the preview stops.
- [ ] Watch LISTEN while a linked pad starts and stops, then repeat with Space. Its
  width and the file-name column must stay fixed. At rest it is dim; while a preview
  or linked pad of this chop plays it is blue with `▶`. Zoom in until the sounding
  position leaves the view: the line is hidden but LISTEN stays blue. After the last
  matching voice stops it becomes dim, even if an effect tail still sounds.
- [ ] On Chop, press `P`, hit an occupied target and press `Enter`. Wait for the preview
  to end. Hit another pad under the replacement question. Check silence, unchanged
  plan and unchanged question. Press `Esc`: nothing is replaced.
- [ ] (by ear) Use the long linked gate on `2-W`. Hold `W` on Chop, press `PgDn`, then
  `F4`, and release `W`. Only the original sound must fade out; the new bank's pad
  must not release. Repeat from Pads to check a hold started outside Chop.
- [ ] (by ear) Hold that gate again, turn bank, open `F4`, then `F10`. Press and release
  a new key `Q`: it stays silent and does not stop the held gate. Release the original
  `W`: only that gate stops. Learn stays open with the same selected item and any
  pending control; no binding is saved. Close learn with `F10`, without `Enter`.
- [ ] On Pads, select a loaded source and press `Ctrl+C`. Turn bank and hit an empty
  target. Check silent placement and unchanged source. Undo with `Ctrl+Z`. Repeat with
  an occupied target. Check its selection and name in the question. Further hits stay
  silent and do not confirm. `Esc` keeps the copy in hand. Hit the target again and
  press `Enter`: only the named target is replaced. One `Ctrl+Z` restores it.
- [ ] On Pads, select a loaded source, press `Ctrl+X`, turn bank and hit a target.
  Check a silent swap and selection of the target. One `Ctrl+Z` restores both pads.
- [ ] On Files, press `Enter` on a sample to hold it, then hit an occupied pad in another
  bank. Check silent targeting and selection of the pad named in the question.
  `Esc` keeps that pad's sound and the file in hand. Cancel the held file before play.
- [ ] On Pads, mark two loaded pads with `M`, pad hits, then `M`. Press `Backspace`.
  Hit a pad under the clear question: no new sound, selection or mark change.
  `Esc` keeps the sounds and marks. Drop the marks with `Esc`; a new hit plays once.

### Controller pass

Repeat with the same starting data. Use controller pads for every target and hit below;
keyboard command keys remain available. Record notes and learned buttons separately if
both are available. A physical test of one does not verify the other.

- [ ] On F2, repeat the PLAY check above with controller notes and a learned pad
  button separately when available. Change bank and release the original gate: PLAY
  follows the actual pad sound, not the selected pad, held flag or brief flash. A new
  hit under question or learn must not light it; a valid older voice can keep it lit.
  This physical pass stays unverified while no controller is available.
- [ ] (by ear) On a fresh 32-slice Chop, hit controller pad 3 in banks 1 and 2.
  Check slice 3 and slice 19, each selected and previewed once as in the keyboard pass.
- [ ] Start the by-hand plan from slice 1 with `P`. Hit controller pad `Q` in bank 1,
  then `W` in bank 2. Check the same two targets and previews, with no pad changes.
  Cancel with `Esc`. Repeat and apply with `Enter`; target hits must not apply it.
- [ ] (by ear) Hit the linked pad with pitch +12 and the low-pass filter on Chop.
  Compare MIDI velocity 127 with `Shift` plus the keyboard pad: the sounds must match.
  Hit lightly and strongly: the light linked-pad hit must be quieter, not a bare preview.
- [ ] Hit the linked pad on Chop in listen and slice edit. Check the same moving green
  line, pitch speed, retrigger, end/release and fixed-width LISTEN badge behavior as
  the keyboard pass. A new hit under question or learn must not start a line; an older
  pad that still sounds can keep its own line. This check stays open without a controller.
- [ ] Open a plan replacement question, wait for preview to end, then hit controller
  pads. Check silence and unchanged question and plan. `Esc` keeps the old targets.
- [ ] (by ear) Hold the long linked gate from controller pad `W` in bank 2 on Chop.
  Turn bank, open `F4` and release the original control. Only the original gate stops.
  Repeat with `F10` open before release. Learn and its selected item stay open; no
  binding is saved or confirmed. A new keyboard key for the same local pad in learn
  must not stop that MIDI hold; only release of the original control stops it.
- [ ] (by ear) Repeat the hold across a bank/screen change, then disconnect that device.
  Check that the original gate fades out without a click. Reconnect and check the same
  binding. If available, test note-off and note-on zero separately; a learned CC button
  must stop its gate with CC0. Record message forms not available on this preset.
- [ ] On Pads, take a copy with `Ctrl+C`. Hit empty and occupied controller targets in
  another bank. Check the same selection, silent placement, replacement question,
  cancellation and one-step undo as in the keyboard pass. Extra hits cannot confirm.
- [ ] Take a file with `F6` and `Enter`, then hit an occupied controller target in another
  bank. Check the selected target, silence and `Esc` as in the keyboard pass. Cancel
  the file. Swap two loaded pads across banks with `Ctrl+X` and a controller target:
  no hit sound, target selected, one `Ctrl+Z` restores both pads.
- [ ] (by ear) Hold a gate from Pads. Open a copy/file replacement or clear question,
  change bank and screen where possible, then release the original control. Only its
  original gate stops; the question stays open. New hits under the question stay silent.
  After cancellation and leaving any held file/copy mode, a new hit plays once.
- [ ] (by ear) On Pads outside these modes, play fast controller hits while the screen
  updates. Check that hits do not wait for the screen and do not sound twice. Turn
  marking on: hits must still sound while they add or remove marks.

### Offline listening and acceptance record

- [ ] (by ear) Listen to `renders/84-input-parity.wav` (5.376 seconds).
  From 0.000 to 2.304 s, the three full-strength hits are keyboard, note and learned
  button: they must sound the same. From 2.304 to 3.840 s, listen to quiet/loud note
  hits. From 3.840 to 5.376 s, listen to quiet/loud learned-button hits. Check silence
  between hits and no clicks, clipping or extra hits. Recreate the WAV with
  `cargo test -p ptsr input_tests::render_84_input_parity -- --nocapture` if needed.

Owner result: checked and accepted change 84 on 2026-11-03 after 2.3 and 2.4;
archive and local signed-off commit explicitly authorized. Individual check results
were not listed. Device/preset: no controller available. Keyboard terminal: not recorded.
Physical note/CC forms: all unverified. Separate WAV listening result: not reported.
The callback benchmark has recorded peaks above the 50% budget target, including
107.46% in the last implementation check. Their cause remains unknown; acceptance
does not close this performance issue. Archive checks must still verify main-spec
scenario coverage, strict validation and the full automated suite.

## 84a — Offline callback timing report

Folder: `changes/archive/2026-11-03-84a-benchmark-report` (`plan/controller-first-patterns`)

The owner accepted this report-only change on 2026-11-03, then explicitly confirmed its
archive with main-spec sync and a local signed-off commit. This is overall acceptance;
individual check results were not reported. Open boxes below do not mean failure or
rejection. Do not start `task run` for these checks. No sound device or controller is
needed. Sound and input behavior are unchanged; no new keyboard, controller or listening
pass is required. No new WAV is provided.
The existing `renders/84-input-parity.wav` keeps the same samples and SHA-256.

- [ ] From the repository folder, run `cargo bench -p engine --bench callback`.
  Read the clock explanation and the unchanged exit checks above the timing groups.
  The command must not start the sampler or require a sound device or controller.
- [ ] Check the empty-call group: 2,000 samples for wall, thread CPU and estimated gap.
  Read the raw means in microseconds and the separate full measurement-envelope mean.
  No overhead is subtracted from the workload results.
- [ ] Check the playback group: 20,000 samples. Each of wall, thread CPU and estimated
  gap has mean, p99, maximum, sample count and count above 50% of the block budget.
- [ ] Check the switch summary: 200 switches. The all-callback group has the same count
  as that summary. In the current workload it is 22,000. The three phases have
  200 first fade blocks, 200 second fade blocks and 21,600 clearing/restore blocks.
  Each phase has the same three timing series and five statistics.
- [ ] Read `maximum target`: `EXCEEDED` must be kept visible for any observed wall or CPU
  maximum above 50%, even if the command succeeds. Success still means only that steady
  wall p99 and per-switch wall peak p99 did not exceed 50%; it is not an all-block pass.
  Equality at 50% passes those exit checks. A zero gap is valid when CPU exceeds wall.
- [ ] Read the gap as an estimate, not proof of a cause. Record any exceedance below.
  Accept the report separately from the still-open slow-core performance issue.

Agent verification on 2026-11-03: the 9 report tests pass, including all seven named
scenarios, exact 50% boundaries, a rare peak, clock errors and raw overhead output.
The full offline benchmark and `task check` passed. In the final implementation check,
steady wall p99 was 12.40%, wall maximum 41.94%, CPU maximum 41.97%, with zero wall/CPU
blocks above 50%. The per-switch wall peak p99 was 10.05%. The full empty-call envelope
mean was 0.445 us. These are one run's observations, not a hard-maximum guarantee.
No engine callback, sound, live load, scheduler setting or runtime dependency changed.
Earlier diagnostic runs showed both scheduling delays and CPU-time exceedances; their
slow-core cause remains open. Physical-controller checks from 84 remain unverified.

Archive verification on 2026-11-03: all seven scenarios are in the main audio-engine
spec. Strict validation, `task check` and the release build passed. This run had one
steady wall block above 50%: wall maximum 92.94%, CPU maximum 36.97%, CPU exceedance
count zero. The report said `wall EXCEEDED; thread CPU WITHIN`, while the unchanged p99
checks passed. The gap maximum was 74.69%; its cause is not established by that estimate.
This result keeps the earlier performance issue open.

Owner result: accepted report 84a on 2026-11-03; archive, main-spec sync and a local
signed-off commit explicitly confirmed. Individual manual checks were not reported, so
their boxes stay open. This accepts the report, not a hard-maximum performance guarantee
or resolution of slow-core CPU spikes. No new live, listening or physical-controller
pass is implied; the earlier controller checks remain unverified.

## 85 — Shared pad input and ordered controller targets

Folder: `changes/archive/2026-11-03-85-input-modes`

The owner accepted 85 on 2026-11-03 and authorized its archive and a local signed-off commit.
Closure of 85 and work on 86 share `86-screen-registry` and one MR by owner request.
Implementation and offline checks are complete. No keys, profiles, saved-data formats or callback code changed. The shared
handler keeps the existing text, picker, F10/F12 and overlapping-hand boundaries.
Recorded MIDI and waveform checks do not mark a physical or listening pass complete.
The final agent screen pass at `1a24909` has 66 captures under ignored `target/ux/85-final/`.
These cover edit/mark, banks, copy/swap/undo and filter/file questions.
They also cover Chop listen/BOUNDS/assign cancellation, FX, F10, all F12 tabs and project text.
Saved TOML comparisons proved undo/cancel and save/restart preservation.
Both starts logged `JACK unavailable`, never `JACK: active`.
PipeWire had no ptsr node before, during or after the run. Both exits printed `UX exit: 0`.
The terminal had no audio or kitty releases. The agent injected no MIDI. The design records fixture
expectation corrections. This is screen evidence, not owner or hardware acceptance.
The owner reported successful keyboard checks on 2026-11-03, then accepted the whole change.
The controller is not available. Only the keyboard boxes below reflect an individual check result.
Controller and WAV checks stay open where no individual result was reported. Open boxes do not mean rejection.

### Prepare

Use a disposable project: these checks load, copy, swap and replace sounds. Prepare
two occupied pads and an empty pad in bank 2. For releases, use the long gate and
20 ms release from section 84, with both FX sends at zero. A keyboard gate needs
kitty key releases. Tmux without them cannot check that part.

For a controller pass, use notes 36–51 or explicit GLOBAL pad bindings. A learned
CC button must send its zero value on release. Record missing velocity/release forms
rather than claiming them. Do not reset a profile for this check. The X-Session's
safe command buttons are not a pad grid. Its left Cue on Files takes a file.
A second device can provide the pad notes. The incomplete SMC-PAD template is not accepted here.

### Keyboard pass

- [x] On Pads, hit a loaded pad. Open edit with `Enter`, then hit a different loaded
  pad: it plays once and edit stays open. Close edit. Turn marking on with `M`, hit
  pads, then open edit again. Marks and sound must coexist. No key assignment changes.
- [x] On Files, switch preview off if it is on. Select a different file, press `Enter`
  and hit an occupied pad at once. The old pad must not play. The question names that target. Hit a second occupied pad: the first question and selection stay. `Esc`
  keeps both old sounds and the file in hand. Hit an empty pad: it loads the file
  without a hit sound. Wait for loading to end, then cancel any hand state before play.
- [x] Take a copy with `Ctrl+C`, place it on an empty pad, and undo with `Ctrl+Z`.
  Swap two loaded pads with `Ctrl+X` and a target hit. Undo the swap.
  Placement is silent. One undo restores the prior target or both swapped pads, including their settings.
- [x] Repeat section 84's bare Chop listen, linked pad, slice edit and both assign
  orders (`P`, then `Tab`). A question and F10 block new keyboard hits.
- [x] (by ear) Hold a linked gate from Chop, change bank, open F4 and release its
  original key. Only its original pad stops. Repeat with F10 open before release.
  A new silent learn key must not steal that older keyboard hold.
- [x] Open the Files filter with `/` and type pad letters. They type text, not hits.
  Repeat pad naming (`Ctrl+N`) and project selection (`Ctrl+P`), then cancel. The
  project picker's first Esc clears typed text. Another Esc closes it. Open and close
  F10 and all F12 tabs without saving a test binding. Save and restart the test project.

### Controller pass, when hardware is available

Use controller pad hits for targets below. Use existing bound commands.
Keyboard commands remain available where a controller has none. Repeat notes and learned CC
buttons separately where possible. Record the device and preset.

- [ ] On Files with preview off, take the selected file with a bound command and hit
  an occupied pad immediately. The result is one silent target question, not the old
  sound. Hit a different occupied pad. It cannot change or answer the first question.
  Esc keeps the old sounds and file in hand. An empty target loads silently.
- [ ] Cancel the held file, then send bank-next and a pad hit quickly. The hit uses
  the new bank once, at its original velocity. Repeat entering and leaving Chop with
  the existing bound commands. Listen and assign keep their normal meaning.
- [ ] (by ear) Hold a long gate. Change bank/screen. Release its original control.
  Repeat under a question and F10. Test note-off, note-on zero, CC zero and disconnect
  where the device supports them. Only the actual original pad stops, without a stuck
  gate. Also retrigger that control quickly around a command and release.
- [ ] (by ear) On Chop, hold a keyboard gate, change bank, then hold the same local
  position from MIDI. Release each in both orders. Each stops its own original pad.
  Repeat MIDI-first, and repeat under a question/F10. A silent new hit steals no hold.
- [ ] (by ear) Outside target modes, play soft and strong hits while the screen updates.
  Each hit sounds once without waiting for the screen. Compare MIDI 127 with a Shift
  keyboard hit. A fixed-127 button cannot check low velocity. Record that limit.
- [ ] If two devices are available, take a file from device A and hit the target from
  B before the screen changes. It is still a silent target. Release older gates from
  both devices during this sequence: their original releases still work.
- [ ] An old MIDI hit delivered after Chop opens must not restart sound, preview or change an assign plan.

### Offline listening

Regenerate the three 85 WAV files with:

```sh
cargo test -p ptsr input_tests::render_85 -- --nocapture
cargo test -p ptsr input_tests::render_84_input_parity -- --nocapture
```

- [ ] (by ear) Listen to `renders/85-input-modes.wav` (2.064 s). The short hits below
  must have one onset and end cleanly. File-target segments are intentional silence.
  Equal-velocity keyboard, note and button segments must sound the same.

| Time, seconds | Segment |
| --- | --- |
| 0.000–0.176 | Keyboard, velocity 80 |
| 0.176–0.352 | Note take-file and occupied target: silence |
| 0.352–0.528 | Direct note, velocity 80, edit and marking open |
| 0.528–0.768 | Deferred note gate, then original release after bank/screen change |
| 0.768–0.944 | CC-button take-file and occupied target: silence |
| 0.944–1.120 | Direct CC button, velocity 80, edit and marking open |
| 1.120–1.360 | Deferred CC gate, then original release after bank/screen change |
| 1.360–1.536 | Direct note, velocity 23 |
| 1.536–1.712 | Direct note, velocity 101 |
| 1.712–1.888 | Direct CC button, velocity 23 |
| 1.888–2.064 | Direct CC button, velocity 101 |

- [ ] (by ear) Listen to `renders/85-mixed-holds.wav` (3.584 s): first notes, then
  learned buttons. Each group has keyboard-first, MIDI-first and rejected-MIDI cases.
  Each original hold must end on its own release. Rejection must not steal the key.
- [ ] (by ear) Listen to `renders/85-late-midi.wav` (about 1.451 s): accepted note,
  rejected note, accepted button, rejected button. Accepted hits have their normal
  Chop-entry hush without a restart. Rejected segments are silent.
- [ ] (by ear) Listen again to `renders/84-input-parity.wav` (5.376 s). Its SHA-256
  remains `f795d105ad3f098d03857203e0eaffed73e782dd1d51bcfdfec233f7d35a4da6`.

Automation checks finite samples, no clipping, intentional silence, one hit and timing
within one sample against timestamped offline oracles. It also tests an older direct
release processed before a deferred retrigger reaches UI: the newer accepted hit keeps
its own release. Existing non-deferred Chop release merging stays unchanged. No formal
memory-model check or exhaustive proof of every queue interleaving is claimed.

Owner keyboard result, 2026-11-03: all checks possible with the keyboard passed.
The owner cannot test a controller at present. Keyboard terminal: not reported.
Device/preset and physical note/CC release forms: unverified. Separate WAV listening result: not reported.
[PR #4](https://github.com/fl64/ptsr/pull/4) is merged into `master` at `fd6a566`.
The owner later accepted 85 and authorized archive and a local signed-off commit on 2026-11-03.
This later decision replaces the earlier archive restriction. It does not report individual WAV or hardware results.
Keep the archive and status update separate from 86 specifications and code in the combined MR.
Callback maximum-budget and slow-core issues remain open. Green p99 benchmark gates do not prove
that every callback meets the 50% target. This change does not investigate that issue.

## 86 — Screen registry and invalid MIDI state

Folder: `changes/archive/2026-11-04-86-screen-registry` (`86-screen-registry` branch)

Implementation is complete through `a186fbb`. The owner accepted 86 on 2026-11-04.
This is overall acceptance, not individual keyboard, physical-controller or WAV listening results.
The owner authorized archive, local signed-off commits and one PR with closure of 85.
Keep the closure commits separate. The owner also requested an ASD-STE100 review of the relevant documents.
The registry keeps existing keys, help, learned assignments and FX controls for valid input.
Invalid shared MIDI state rejects new actions without a fallback screen. Original releases still work.
No audio callback, sound algorithm, profile or saved-data format changed.

### Automated evidence (2026-11-04)

The agent reviewed the full 86 diff from `a1e4829` through `a186fbb`.
Existing behavior assertions remain. Focus-read assertions now check the `Result` representation.
Implementation commits are `0cebfed` (registry), `719d90b` (bindings), `44eb4d6` (help and learning) and `a186fbb` (invalid MIDI state).

All nine delta scenarios have tests with matching names in `crates/app/src/input_tests.rs`.
The separate delta check reported `Delta coverage: 9/9 named tests.`
The main-spec coverage test passed too. It does not inspect active deltas.
Raw MIDI uses the production profiles, dispatcher and App. Only state corruption uses test-only injection.
Tests cover unknown codes, invalid banks, reserved bits, original releases, full queues, project epochs and recovery with earlier UI work pending.

Commands completed successfully:

```sh
openspec validate --all --strict --no-interactive
cargo test -p ptsr --test scenarios
cargo test -p ptsr input_tests::render_85 -- --nocapture
task check
task build
```

Strict validation reported `Totals: 16 passed, 0 failed (16 items)`.
The main-spec check reported one passed test. The explicit render check reported three passed tests.
`task check` passed formatting, Clippy, all workspace tests and the benchmark.
Its input, App and TUI test summaries were:

```text
test result: ok. 80 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 670 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 36.36s
test result: ok. 101 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s
```

The release build passed. Local logs are `target/86-step-4.1-check.log`,
`target/86-step-4.1-build.log`, `target/86-step-4.1-renders.log` and `target/86-step-4.1-delta-coverage.log`.
These ignored artifacts do not replace this record or owner checks.

The recorded benchmark in `target/86-step-4.1-check.log` reported:

```text
32 voices with pitch, envelope, filter, drive and sends, reverb and delay, block 128 @ 48000 Hz, budget 2.667 ms: mean 7.83 %, p99 10.22 %, max 30.83 %
  wall: mean 7.83 %, p99 10.22 %, max 30.83 %, samples 20000, >50 % 0
  thread CPU: mean 7.84 %, p99 10.15 %, max 29.07 %, samples 20000, >50 % 0
project switch: 200 switches, 22000 callbacks, mean 0.17 %, callback p99 7.04 %, switch peak p99 9.43 %, max 10.53 %
```

Playback and switch groups had zero wall/CPU blocks above 50% in this run.
This is one observation, not a maximum-budget guarantee. Earlier callback peaks and slow-core CPU spikes remain unresolved.
No scheduler or system setting changed. A green p99 gate does not close the timing issue.

The three existing WAV checks passed finite-sample, clipping, silence and timing checks against timestamped waveform references.
The regenerated files also matched their SHA-256 values from before this verification run:

```text
01fafeb65b32d3bceb00210eb40d4be904dc3ea4d7b029068f09eb07bcf3743f  renders/85-input-modes.wav
22685916966160ee50774ee80245c3644ab6dae301f05137f15e74eea8d97bce  renders/85-mixed-holds.wav
b80bdb052a43559e435bcd72c456323b84fb27b62df1b5cc54e68e5fdce08514  renders/85-late-midi.wav
```

No new WAV is necessary. The agent did not run the sampler live, hear sound or test physical MIDI hardware for 86.
Keyboard, controller and listening results for 86 remain unreported. Automation does not mark those checks complete.

### Documentation handover verification (2026-11-04)

The documentation check passed strict OpenSpec validation, `task check` and `task build`.
The validation and App test reported:

```text
Totals: 16 passed, 0 failed (16 items)
test result: ok. 670 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.10s
```

The playback wall maximum was 20.46%, and its thread CPU maximum was 19.81%.
The switch wall maximum was 9.08%, and its thread CPU maximum was 9.11%.
All workload groups reported zero wall and CPU blocks above 50% in this run.
This observation does not close the earlier timing issue. The three 85 WAV hashes still match the values above.
Local logs use the prefix `target/86-step-4.2-`: `check.log`, `build.log`, `openspec.log` and `ste.log`.

STE review used STE-flavored descriptions and Strict procedures. The lint found zero hard violations and four passive-voice suggestions.
The suggestions concern `is recorded` in the status descriptions. The actor is irrelevant there, so that wording stays.
The new owner procedures have no lint findings. This check does not establish full ASD dictionary compliance.

### Prepare the owner checks

Use a disposable project because these checks change pad and effect settings.
Start the program from the repository folder with `task run`.
Keep the existing keyboard and controller maps. Do not reset or replace saved assignments.
The keys below are defaults. If your map replaces a key, use its existing replacement and record it.

1. Open Files with `F6`.
2. Select a sample at least 20 seconds long.
3. Press `Enter` to take the file in hand.
4. Hit an empty pad to load the file.
5. Wait for loading to finish.
6. Open Chop with `F3`.
7. Press `Y` to make two slices from a fresh sample.
8. Press `P` to open the assignment plan.
9. Hit two empty pads to select the targets.
10. Press `Enter` to apply the plan.

For the release checks, use one linked slice as a long gate.
On Pads, select that pad. Press `H` until its mode is gate (`⇥`).
Set its release to 20 ms on ENV. Set both FX sends to zero.
Use a terminal with the kitty keyboard protocol. A terminal without key releases cannot check keyboard gate release.
For the FX sound check, use another loaded pad with a nonzero send to the selected block.

For controller commands, use the X-Session Pro with its existing assignments.
Its default command buttons are not pad controls.
For pad holds, use a second device with notes 36–51 or existing GLOBAL pad bindings.
Use keyboard commands where the controller has no assignment.
Record unavailable devices, bindings and release forms instead of marking them complete.
The incomplete SMC-PAD template is not a verified hardware profile.

### Keyboard pass

- [ ] Open Pads (`F2`), Chop (`F3`), FX (`F4`), Files (`F6`) and Settings (`F12`) in sequence.
  Check the screen title after each key. Open `F1` on each screen.
  Check that the full help names that screen's controls. Close help with `F1`.
  Check the bottom help, including modifier hints and the current Settings tab.
- [ ] On Pads, select the loaded pad. Press `Enter` to open edit.
  Check pages with `Tab` and `Shift+Tab`. Select parameters with `I` and `K`.
  Change a value with `J` and `L`. Check that the bank does not change.
  Close edit with `Esc`.
- [ ] Open the linked slice on Chop with `F3`. Press `Enter` to open slice edit.
  Check that only the `BOUNDS` page is available. Close edit with `Esc`.
- [ ] Open FX with `F4`. Select each block with `Tab` and `Shift+Tab`.
  Select parameters with `I` and `K`. Change values with `J` and `L`.
  Check that these controls need no separate edit mode.
- [ ] Open `F10` on Pads, Files, Chop and FX in separate runs.
  Check the corresponding group: `PADS`, `FILES`, `CHOP` or `FX`.
  Close learn with `F10` without saving a binding.
  Repeat with Pads edit and Chop edit open. Check `PADS · EDIT` and `CHOP · EDIT` respectively.
  Check that FX has no separate edit group.
- [ ] On Settings, press `F10`. Check that screen learn does not open.
  Open the MIDI tab for its existing learning list. Do not save a test binding.
- [ ] Open `F12` from Pads, Chop, Files and FX in separate runs.
  Open the MIDI tab. Check that the list shows the corresponding screen group expanded.
  Check the group order: `GLOBAL`, `PADS`, `PADS · EDIT`, `SETTINGS`, `FILES`, `CHOP`, `CHOP · EDIT`, `FX`.
- [ ] (by ear) On Chop, hold the long linked gate's pad key.
  Change bank with `PgDn`. Open FX with `F4`. Release the original key.
  Check that only the original pad stops. The new bank must not select a different release target.
- [ ] Restart the program. Check an existing custom key or explicit `none` entry if your map has one.
  Check that its previous effect remains. Do not create or replace an owner assignment for this check.

### Controller pass, when hardware is available

Use the existing map. The physical names below describe the X-Session defaults from change 82.
If your map replaces a default, use its existing assignment and record the difference.

- [ ] Press left Headphone for Files and right Headphone for FX if your map retains those defaults.
  Check that each opens its existing screen.
- [ ] On Pads, press left Cue to open edit if that assignment is present.
  Check the existing page buttons and numbered knobs 1–3.
  Close edit with a Cue button that already has this role.
- [ ] On FX, select both blocks with the existing page buttons.
  Check the same numbered parameter knobs on each block, without another edit mode.
  Check any saved FX assignment with its existing priority. Do not learn a replacement for this check.
- [ ] Open `F12` from Pads, Chop, Files and FX in separate runs.
  Use an existing controller assignment for entry if available. Otherwise, use `F12` on the keyboard.
  Open the MIDI tab. Check the corresponding expanded group and the group order from the keyboard pass.
- [ ] Test one saved screen assignment and one saved GLOBAL assignment.
  Restart the program. Check that both retain their previous roles without learning again.
- [ ] (by ear) On Chop, hold the long linked gate from a pad controller.
  Change bank. Open FX. Release the original control.
  Check that only the original pad stops. Repeat the sequence with a device disconnect instead of release.
  Check that the original gate stops without a stuck note.
- [ ] Repeat the release check with note-off, note-on zero and learned-button zero where the device supports them.
  Record the available forms. A command-only device cannot check pad holds.

Invalid internal state is an automated fault test. Do not change internal state or configuration to reproduce it by hand.

### Listening pass

These are the existing regression references, not new sound features.
Section 85 gives the segment times and the command to regenerate the files.

- [ ] (by ear) Listen to `renders/85-input-modes.wav` (2.064 s).
  Check one onset per accepted hit, intentional silence for file targets and clean gate stops.
  Check that equal-velocity keyboard, note and button hits sound the same.
- [ ] (by ear) Listen to `renders/85-mixed-holds.wav` (3.584 s).
  Check that each original hold ends on its own release. A rejected hit must not steal another hold.
- [ ] (by ear) Listen to `renders/85-late-midi.wav` (about 1.451 s).
  Check the normal Chop-entry fade without a second hit. Rejected segments must stay silent.

During the live keyboard and controller passes, listen for missing releases, duplicate hits and changed FX response.
No new WAV is necessary because the sound algorithm does not change.

Owner result, 2026-11-04: overall acceptance of 86. Individual keyboard and listening results remain unreported.
Keyboard terminal and custom keys: not reported.
Controller device, preset, assignments and physical release forms: unverified. Separate WAV listening: not reported.
The owner explicitly authorized archive, local signed-off commits and PR publication. All individual manual boxes stay open.
Earlier callback peaks and slow-core CPU spikes remain open.

### Archive verification (2026-11-04)

`openspec archive 86-screen-registry --yes` added four requirements and nine scenarios to the two main specs.
The archive comparison preserved all earlier main-spec text and found named tests for all nine scenarios.
Strict validation, main-spec scenario coverage, `task check` and `task build` passed.
The validation and App test reported:

```text
Totals: 15 passed, 0 failed (15 items)
test result: ok. 670 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.58s
```

The callback benchmark passed its existing p99 exit checks but exceeded the maximum-budget target:

```text
paired timing: steady playback
  wall: mean 7.99 %, p99 9.36 %, max 93.23 %, samples 20000, >50 % 2
  thread CPU: mean 7.99 %, p99 9.28 %, max 93.30 %, samples 20000, >50 % 2
  maximum target (50 %): wall EXCEEDED; thread CPU EXCEEDED
```

The switch workload reported zero wall and CPU blocks above 50%.
These results keep the earlier maximum-budget and slow-core issues open. They do not identify a cause.
Local logs use the prefix `target/86-closure-`: `check.log`, `build.log`, `openspec.log` and `scenarios.log`.
No sampler live run, owner listening or physical-controller result comes from these checks.

### PR preparation checks (2026-11-04)

The separate documentation edit simplifies English without changing behavior or scenario titles.
The main and archived 85 requirements match. All unrelated input requirements remain unchanged.
The ASD-STE100 review uses STE-flavored descriptions and Strict procedures.
The selected text has zero hard lint findings. Remaining advice concerns passive voice and one necessary temporal condition.
These results do not prove full ASD dictionary compliance.

`task check`, main-spec scenario coverage and `task build` passed again.
The App test reported:

```text
test result: ok. 670 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 35.87s
```

This run reported steady wall p99 9.28%, maximum 28.32%, and CPU p99 9.26%, maximum 16.15%.
Both steady and switch workloads had zero blocks above 50%.
This observation does not close the earlier maximum-budget or slow-core issue.

The current OpenSpec 1.14.3 rejects specification warnings in strict mode.
These warnings concern requirements longer than 500 characters and placeholder Purpose sections.
An isolated copy of the unchanged `b54856d` specifications gives the same strict result as the edited documents:

```text
Totals: 3 passed, 12 failed (15 items)
OpenSpec strict exit: 1
```

Validation without strict mode reports:

```text
Totals: 15 passed, 0 failed (15 items)
OpenSpec standard exit: 0
```

At that stage, strict validation remained a merge blocker.
The later specification cleanup resolved these warnings without changing scenario text.
Evidence: `target/86-new-cli-baseline.log` and the logs with prefix `target/86-pr-`.
No new owner, listening or hardware result comes from these checks.

## 86b — Save retry and learning from F12

Implementation and automated checks are complete. The owner accepted 86b without the manual passes.
Keyboard, physical-controller and listening results remain unverified. Their checkboxes below do not block archive.
A failed write keeps pending work and pauses automatic saving, including saving on exit.
A successful manual save clears the warning and permits autosave after the next edit.
Read protection stays active for damaged, unreadable and newer session files.
The F12 MIDI tab now lists `save session` under GLOBAL.
Keys, factory assignments, saved formats and sound processing stay unchanged.

### Recorded implementation checks

`task check` exited 0 after the save-learning amendment. Its App, input and TUI suites reported:

```text
test result: ok. 681 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 44.97s
test result: ok. 80 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 102 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.94s
```

Rust tests name all 21 active delta scenarios.
This source check supplements execution. It does not prove owner or hardware acceptance.
The tests cover scratch and projects, saved bytes, read protection, temporary-path collisions, missing folders and paused exit saving.
Controlled-time tests cover notice expiry, replacement notices and resumed autosave.
Screen tests cover 60×24, 100×26 and 252×60, with and without audio, including modal help.
The new recorded-input test learns note and CC buttons through F12 instead of injecting the save assignment.
It tests pending confirmation, Esc, configuration reload, five input screens, repeated refusals and a successful retry.

`task build` passed and produced `target/release/ptsr`. `task openspec:check` reported:

```text
Totals: 16 passed, 0 failed (16 items)
```

The implementation benchmark passed its p99 exit checks but exceeded the wall maximum target:

```text
paired timing: steady playback
  wall: mean 17.87 %, p99 24.85 %, max 91.71 %, samples 20000, >50 % 1
  thread CPU: mean 17.88 %, p99 24.77 %, max 32.19 %, samples 20000, >50 % 0
  maximum target (50 %): wall EXCEEDED; thread CPU WITHIN
```

Project-switch maxima were 27.13% wall and 27.16% thread CPU, with no blocks above 50%.
An earlier 86b run reached 88.34% wall and 88.39% thread CPU, with one block above 50% for each clock.
The cause remains unknown. These results do not close the maximum-budget or slow-core issue.
Local evidence: `target/86b-learn-code-final-check.log`, `target/86b-owner-build.log`, `target/86b-learn-code-openspec.log` and `target/86b-owner-scenarios.log`.
The earlier overrun remains in `target/86b-step3-final-check.log`.
The final handover `task check` also exited 0, with 681 App tests and 102 TUI tests passing.
Its steady maxima were 55.54% wall and 35.12% thread CPU. One wall block exceeded 50%.
Project-switch maxima were 15.13% wall and 15.15% thread CPU, with no blocks above 50%.
Evidence: `target/86b-owner-final-check.log`. The maximum-budget acceptance remains open.
The first save-learning test used the wrong project filename. The corrected fixture uses `project.toml` and passes.
A subsequent check stopped on formatting. `task fmt` corrected that line before the successful full run.

### Prepare an isolated owner run

Owner procedure only: this run uses normal sound and reads connected MIDI devices.
The agent must not run these commands. Agent live runs require the sound isolation in `AGENTS.md`.
These temporary folders protect existing projects, scratch work and controller assignments.
Keep this shell open. Run these commands from the repository folder after `task build` and `task check`:

```sh
check_root="$(mktemp -d "$PWD/target/86b-owner.XXXXXX")"
mkdir -p "$check_root/config/ptsr" "$check_root/data" "$check_root/state" "$check_root/projects" "$check_root/samples"
cp renders/85-input-modes.wav "$check_root/samples/check.wav"
printf '[paths]\nprojects = "%s/projects"\nsamples = "%s/samples"\n' "$check_root" "$check_root" > "$check_root/config/ptsr/config.toml"
printf 'Owner check folder: %s\n' "$check_root"
run_save_check() {
  env XDG_CONFIG_HOME="$check_root/config" XDG_DATA_HOME="$check_root/data" XDG_STATE_HOME="$check_root/state" ./target/release/ptsr
}
run_save_check
```

In a second terminal, set `check_root` to the exact printed folder before using the commands below.
Do not use an existing project folder for these refusal tests.
Do not quit or switch projects with pending work. This change adds no quit guard or safe project-switch guard.
Those transitions belong to 86c. The loss risk on exit still exists.

### Keyboard pass

- [ ] Start in scratch. Open Files with `F6`, select `check.wav` and press `Enter`.
  Hit pad `1` to load it. Wait for loading to finish.
  Press `Ctrl+S` twice. Each press must show `Session saved` for five seconds.
- [ ] Set `session_file="$check_root/data/ptsr/session.toml"` in the second terminal.
  Copy its saved bytes and occupy only its temporary path:

  ```sh
  cp "$session_file" "$check_root/last-saved.toml"
  mkdir "$session_file.tmp"
  ```

  On Pads, press `]` to change master volume. Check `Unsaved`, then `Save failed` after the idle save.
  Press `Ctrl+S` twice. Each request must show a current refusal. Wait more than five seconds.
  Check that `Save failed` remains after the notice ends. Compare the saved bytes:

  ```sh
  cmp "$session_file" "$check_root/last-saved.toml"
  ```

  No output and exit 0 mean the bytes match.
- [ ] Open Pads, Files, Chop, FX and Settings in separate passes. Return to Pads before opening Chop.
  Keep a loaded pad selected for Chop. Check `Save failed` at 60×24 and at a larger terminal size.
  Open and close `F1`. Check that help and notices retain their places.
  On Pads, select an empty pad and press `F3`. Its refusal notice must not remove `Save failed`.
- [ ] Remove only the test directory with `rmdir "$session_file.tmp"`. Wait two seconds without a manual save.
  Run the same `cmp` command. The saved bytes must still match because automatic retries remain paused.
  Press `Ctrl+S`. Check `Session saved` and no warning. Press `]` again.
  After two idle seconds, check that `Unsaved` clears without a success notice.
  Quit with `Ctrl+Q`, run `run_save_check` again and check the saved master volume.
- [ ] Save scratch before the project pass. Press `Ctrl+P`, type `retry` and press `Enter` to create the disposable project.
  Load `check.wav` again. Save it. Set `session_file="$check_root/projects/retry/project.toml"` in the second terminal.
  Repeat the temporary-path refusal, byte comparison, manual retry and later autosave checks above.
- [ ] With the disposable project clean, rename its folder in the second terminal:

  ```sh
  mv "$check_root/projects/retry" "$check_root/projects/retry-away"
  ```

  Press `]`, then `Ctrl+S` twice. Each request must refuse. Check that the old folder does not reappear.
  The window title must drop the project name. Restore the same folder:

  ```sh
  mv "$check_root/projects/retry-away" "$check_root/projects/retry"
  ```

  Press `Ctrl+S`. Check that the retry succeeds before closing the program.

### Controller pass, when hardware is available

Use the isolated configuration above. Record the device, preset and whether its button sends Note or CC messages.
Choose an unused button. Do not replace an existing performance control to test saving.

- [ ] Open `F12`. Use `Tab` to select MIDI. Select GLOBAL with the arrows and press `Enter` to open it.
  Select `save session`. Press `Enter`, then the unused controller button.
  Check the pending assignment on that row. Press `Esc` and check that no assignment changes.
- [ ] Repeat learning. Press `Enter` on the pending assignment to save it. Save all session work before exit.
  Restart with `run_save_check`. Check that the row retains the assignment.
- [ ] Press the learned button on Pads, Files, Chop, FX and Settings. Check `Session saved` on each screen.
  Return to Pads before opening Chop and keep a loaded pad selected.
- [ ] Repeat the keyboard pass's temporary-path refusal and retry with the learned button instead of `Ctrl+S`.
  Check each repeated refusal, the persistent warning, successful retry and silent autosave after the next edit.
  Keep keyboard and controller results separate. Repeat with the other message form only if the hardware supports it.

### Read-protection pass

Use only the disposable `retry` project. Save and close it before each file edit.
Keep a known good copy outside the project folder:

```sh
cp "$check_root/projects/retry/project.toml" "$check_root/good-project.toml"
printf 'version = [\n' > "$check_root/projects/retry/project.toml"
```

In the original shell, start the program with `run_save_check` after each file edit below.

- [ ] Press `Ctrl+S` twice. Each request must explain the read refusal, not show `Session saved`.
  Close the program. Check that the damaged file still contains exactly the text from the command.
- [ ] Replace that disposable file with `printf 'version = 99\n' > "$check_root/projects/retry/project.toml"`.
  Start again. Repeat the refusal and unchanged-file checks. A manual retry must not remove read protection.
- [ ] Close the program. Restore it with `cp "$check_root/good-project.toml" "$check_root/projects/retry/project.toml"`.
  Start again and check that ordinary saving works. Do not change a protected file while that session remains open.

Isolated automated tests also cover unreadable files and protected scratch sessions. The agent did not run these cases live.

### Listening and acceptance

No new render is necessary: sound processing stays unchanged.
The copied `renders/85-input-modes.wav` is an existing test sample, not a new 86b sound feature.

- [ ] (by ear) Play the same loaded pad before a refusal and after successful retry. Check for new clicks or changed playback.
- [ ] (by ear) After restart, play the saved pad. Check the retained settings against the values before exit.
- [ ] Report keyboard, physical-controller and listening results separately, including unavailable controls.

No live sampler run or physical-controller result comes from the automated checks above.
The owner authorized continuation without all manual passes. Keep their checkboxes unchecked, not passed.
`openspec archive 86b-save-retry --yes` archived 86b as `2026-11-05-86b-save-retry`.
The archive added three requirements and modified three requirements across four main specs.
Strict validation initially rejected the length of the saving requirement. Its shorter text keeps the same rules and scenarios.
After that text edit, strict validation reported `Totals: 15 passed, 0 failed (15 items)`.
Main-spec scenario coverage and `task check` passed after archive.
The archive run's steady maxima were 28.97% wall and 16.40% thread CPU, with no blocks above 50%.
This run does not close earlier overruns. Evidence: `target/86b-archive-check.log` and `target/86b-archive-final-openspec.log`.
Next: activate 86c, followed by 86d. Change 87 remains outside this work.
Callback maximum-budget acceptance remains open.

## 86c — Keep work during project transitions

The owner accepted 86c with its recorded limits and without the manual passes.
Live keyboard, physical-controller and listening results remain unverified. Their checkboxes stay unchecked.
Pending work now saves before another project opens or a new project starts. This includes scratch work.
A refusal keeps the picker selection or typed name, source edits, undo and pending timestamp.
Selecting the current project, including a folder alias, closes the picker without a save or reload.
A target read refusal keeps the current project and the last-project setting unchanged.
A target creation refusal after source saving keeps that source open with its work safely saved.
No new keys, file formats, profiles or sound-processing rules enter this change. There is still no quit guard.

### Recorded automated checks

Seven new recorded-input tests use production keyboard input and raw MIDI through the profile and dispatcher.
They cover named projects and scratch, open and create, repeated refusals, restored folders and retry.
Target faults include damaged, newer, unreadable and missing sessions. An occupied directory provides the unreadable fixture.
The same-folder check includes a symbolic link, retained undo, loader generations, sound identity and an unchanged audio epoch.
Picker arrows, typed names and Enter remain keyboard operations. These tests do not prove controller-only navigation.
Rust tests name all 11 delta scenarios. The existing late-loader and project-isolation tests also passed.

`task check` exited 0. Selected suite results:

```text
test result: ok. 688 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 68.08s
test result: ok. 80 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 102 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.83s
```

`task build` passed and produced `target/release/ptsr`. `task openspec:check` reported:

```text
Totals: 16 passed, 0 failed (16 items)
```

The callback benchmark exceeded the maximum target despite the successful command exit:

```text
paired timing: steady playback
  wall: mean 17.20 %, p99 38.24 %, max 959.97 %, samples 20000, >50 % 104
  thread CPU: mean 16.58 %, p99 37.13 %, max 149.04 %, samples 20000, >50 % 10
  maximum target (50 %): wall EXCEEDED; thread CPU EXCEEDED
```

Project-switch maxima were 15.14% wall and 15.16% thread CPU, with no blocks above 50%.
The cause of the steady overruns remains unknown. No sound-processing code changed.
Callback maximum-budget acceptance stays open. Do not treat the exit code as maximum-budget acceptance.
The full check runs tests in parallel and can occupy several CPU cores. It does not start the live sampler.

Before the code edit, a specification check failed in `tests::loading_a_long_file_during_play` with `the loader did not answer`.
The isolated test then passed in 1.01 seconds. A full repeat passed 681 App tests in 99.78 seconds.
This repeat does not establish the cause of the earlier timeout.
The first code check stopped on Clippy's ``this `if` statement can be collapsed``.
The corrected condition passed the full check without an exception.
Startup protection fixtures now use the startup load path. The browser fixture switches away before reopening the original project.
Neither fixture depends on the removed interactive empty-session fallback or same-folder reload.

Local evidence: `target/86c-code-final-check.log`, `target/86c-final-build.log` and `target/86c-final-openspec.log`.
Timeout evidence: `target/86c-spec-check.log`, `target/86c-long-load-probe.log` and `target/86c-spec-repro-check.log`.
The earlier Clippy result remains in `target/86c-code-check.log`.

### Prepare a disposable owner run

Owner procedure only: this run uses normal sound and reads connected MIDI devices.
The agent did not run it. Agent live runs require the sound isolation in `AGENTS.md`.
Run these commands from the repository folder. Keep this terminal open.

```sh
check_root="$(mktemp -d "$PWD/target/86c-owner.XXXXXX")"
mkdir -p "$check_root/config/ptsr" "$check_root/data/ptsr" "$check_root/state" "$check_root/samples"
cp renders/85-input-modes.wav "$check_root/samples/check.wav"
printf '[paths]\nprojects = "%s/projects"\nsamples = "%s/samples"\n' "$check_root" "$check_root" > "$check_root/config/ptsr/config.toml"
printf 'version = 3\nbrowser_dir = "%s/samples"\n' "$check_root" > "$check_root/data/ptsr/session.toml"
for project in A B; do
  mkdir -p "$check_root/projects/$project/samples"
  printf 'version = 3\nbrowser_dir = "%s/samples"\n' "$check_root" > "$check_root/projects/$project/project.toml"
done
printf 'Owner check folder: %s\n' "$check_root"
run_transition_check() {
  env XDG_CONFIG_HOME="$check_root/config" XDG_DATA_HOME="$check_root/data" XDG_STATE_HOME="$check_root/state" ./target/release/ptsr
}
run_transition_check
```

In a second terminal, set `check_root` to the exact printed folder.
Use only these disposable folders for file changes. Do not quit with pending work.

### Keyboard pass

- [ ] Start in scratch. Press `F6`. Select `check.wav` and press `Enter`. Hit pad `1` to load it.
  Wait for loading. Press `H` on Pads to change its mode.
  Immediately press `Ctrl+P`. Select `B` with the arrows. Press `Enter`.
  Check that `B` opens and the scratch file changes before the transition.
  Count this as an immediate-switch pass only if the work remained unsaved before Enter.
- [ ] Use `Ctrl+P` to open `A`. Load `check.wav` onto pad `1` through Files.
  On Pads, press `H`. Press `F3`. Move the cursor away from both ends with `Right`.
  Press `M` to place a cut. Press `P`. Hit pad `2`. Press `Enter` to apply the slice assignment.
  Press `F2`. Open `B` through the picker. Open `A` again.
  Check the retained pad mode. Select the assigned pad and press `F3`. Check the saved cut.
- [ ] With `A` open, occupy its temporary path from the second terminal:

  ```sh
  mkdir "$check_root/projects/A/project.toml.tmp"
  cp "$check_root/projects/A/project.toml" "$check_root/source-saved.toml"
  ```

  On Pads, press `]`. Wait for `Save failed`.
  Open the picker. Select `B`. Press `Enter` twice.
  Check that each refusal keeps `A`, the same selection and the pending edits. The picker must remain open.
  Press `Esc`. Open the picker again. Type `C`. Press `Enter` twice.
  Check that the name stays `C` and no `C` folder appears. Check the saved source bytes:

  ```sh
  cmp "$check_root/projects/A/project.toml" "$check_root/source-saved.toml"
  test ! -e "$check_root/projects/C"
  rmdir "$check_root/projects/A/project.toml.tmp"
  ```

  Both checks must exit 0. Press `Enter` again. Check that saving succeeds and `C` opens.
- [ ] Open `A` again. Occupy its temporary path. Change the master volume. Wait for `Save failed`.
  Open the picker and select `A`. Press `Enter`.
  Check that the picker closes without removing the pending edits or warning.
  Press `Ctrl+Z`. Check that it reverses the last edit.
  Remove the temporary directory. Press `Ctrl+S` before continuing.
- [ ] With `A` open, preserve the target bytes and replace only the disposable `B` session:

  ```sh
  cp "$check_root/projects/B/project.toml" "$check_root/target-saved.toml"
  printf 'version = 99\n' > "$check_root/projects/B/project.toml"
  ```

  Open the picker. Select `B`. Press `Enter` twice.
  Check the target-read refusal. Check that `A` remains open and the selection stays on `B`.
  Restore the target with the command below. Press `Enter` again. Check that `B` opens.

  ```sh
  cp "$check_root/target-saved.toml" "$check_root/projects/B/project.toml"
  ```

### Physical-controller pass

- [ ] On F12, open the MIDI tab. Learn a button for GLOBAL `project picker`. Press Enter to save its assignment.
  Learn separate buttons for Pads `pad mode` and Chop `cut at the cursor`.
  Use those buttons for the edits and picker entry in the keyboard pass above.
  Keep picker arrows, typed names and Enter on the keyboard. Controller-only picker navigation is not part of 86c.
- [ ] Repeat a source-save refusal and a successful retry. Check that the learned picker button opens the same retained work.
  Report unavailable controls instead of marking their checks passed.

### Listening and acceptance

No new render is necessary. Existing renders remain local:

- [ ] Listen to `renders/79-project-switch.wav`. Check that the old project stops and its sound returns when reopened.
- [ ] Listen to `renders/11-switch-nothing-leaks.wav`. Check that the previous project leaves no sound in the next one.
- [ ] In the disposable live run, play a pad before a refused transition. Check that refusal does not stop or replace its sound.
- [ ] Report live keyboard, physical-controller and listening results separately. All remain unverified until the owner reports them.

The owner authorized archive and continuation to 86d. Acceptance does not close the callback-budget issue.
A separate agent investigates its cause. No cause or correction is established here.
Change 87, a quit guard and callback-budget changes remain outside 86c.


## 86d — Keep every chop variant after a project move

Project saving now converts older variant keys with the same path rules as pads and current chops.
Internal sample paths move with the project. External absolute paths stay unchanged. The program does not copy samples.
Full older states retain unassigned cuts, cut IDs, epochs, next IDs and pad settings.
Projects without saved older states keep the existing pad-bound fallback.
Already stale absolute keys stay unchanged. The program does not guess replacement samples.
The saved format, keys, profiles and sound processing stay unchanged.
The owner accepted 86d on 2026-11-05 with the recorded limits and authorized archive.
The live checks below remain unverified. Overall acceptance does not establish their results.

### Automated coverage and local evidence

The tests use real WAV files and the production project, save and input routes.
Keyboard and raw MIDI runs cover three variants of one sample, including two older states with unassigned cuts.
They cover internal and external paths, whole-folder relocation, a second save/open and isolated bound edits.
They compare complete states and pad settings. External sample bytes stay unchanged and no copy appears in the project.
Compatibility tests cover absent older states and stale absolute keys without a format change.
Rust tests name all eight delta scenarios.

The relocation regression fails at the older-variant stored key without the correction.
With the correction, matching slice renders agree sample for sample before and after the move.
Automatic checks cover finite samples, clipping, leading/trailing silence and hit onset within one sample.
The offline fixture advances real callbacks until the project-ready marker. It uses no sleep or live sound device.

Commands: `task check`, `task build` and `task openspec:check`.
Keep build directories separate between worktrees. Check that both portable-chop tests run in the App suite.
A Debug package rebuild does not refresh the Release callback benchmark. Check both profiles when replacing shared artifacts.
Local evidence stays under `target/`, including `86d-code-check.log`, `86d-clean-check.log` and `86d-clean-build.log`.
Keep raw CSV files, detailed machine logs and identifying local paths outside Git.
The owner permits anonymized verification summaries in documents and commit history.
A successful command does not close the callback maximum-budget issue. No performance correction enters 86d.
Recorded input tests do not establish live keyboard, physical-controller or listening results.

### Prepare a disposable owner run

Owner procedure only: this run uses normal sound and reads connected MIDI devices.
The agent did not run it. Agent live runs require the sound isolation in `AGENTS.md`.
Run these commands from the repository folder. Keep this terminal open.

```sh
check_root="$(mktemp -d "$PWD/target/86d-owner.XXXXXX")"
mkdir -p "$check_root/config/ptsr" "$check_root/data/ptsr" "$check_root/state" "$check_root/samples" "$check_root/projects/Before/samples"
cp renders/85-input-modes.wav "$check_root/projects/Before/samples/check.wav"
cp renders/85-input-modes.wav "$check_root/samples/external.wav"
cp "$check_root/samples/external.wav" "$check_root/external-before.wav"
printf 'version = 3\n' > "$check_root/projects/Before/project.toml"
printf '[paths]\nprojects = "%s/projects"\nsamples = "%s/samples"\n' "$check_root" "$check_root" > "$check_root/config/ptsr/config.toml"
printf 'version = 3\nbrowser_dir = "%s/samples"\n' "$check_root" > "$check_root/data/ptsr/session.toml"
printf 'Owner check folder: %s\n' "$check_root"
run_portable_check() {
  env XDG_CONFIG_HOME="$check_root/config" XDG_DATA_HOME="$check_root/data" XDG_STATE_HOME="$check_root/state" ./target/release/ptsr
}
run_portable_check
```

Use only these disposable folders. Do not quit with pending work.

### Keyboard pass

- [ ] Press `Ctrl+P`. Select `Before` with the arrows. Press `Enter`.
  Press `F6`. Select `check.wav`. Press `Enter`. Hit pad `1` to load the whole file. Wait for loading.
- [ ] Press `F3`. Press `Y` twice for four slices. Press `P`. Hit pad `2`. Press `Enter` to apply only that assignment.
  Press `Y` once for eight slices. Press `P`. Hit pad `3`. Press `Enter`.
  Press `Y` once for sixteen slices. Press `P`. Hit pad `4`. Press `Enter`.
  Do not assign the remaining slices. These cuts must survive even though no pad uses them.
- [ ] Press `F2`. Hit pad `2`, then press `F3`. Check the four-slice variant with all three interior cuts.
  Repeat with pad `3` for eight slices and pad `4` for sixteen slices.
  Press `F2` before selecting each pad. Record their visible bounds.
- [ ] Press `F6`. From `Before/samples`, press `Left` three times to reach the printed check folder.
  Press `/`. Type `samples`. Press `Enter` to close text input. Press `Right` to enter that folder.
  Select `external.wav`. Press `Enter`. Hit pad `Q` to load this external file. Wait for loading.
  Press `Ctrl+S`. Check `Session saved`. Press `Ctrl+Q` to quit.
- [ ] In the same shell, move the whole project folder and restart:

  ```sh
  mv "$check_root/projects/Before" "$check_root/projects/After"
  run_portable_check
  ```

  Press `Ctrl+P`. Select `After`. Press `Enter`.
  Repeat the pad `2` / `3` / `4` F3 checks. Every cut must remain, not only the assigned slice bounds.
  Press `F2`. Play pad `Q`. Check that the external sample still loads and sounds.
- [ ] Press `Ctrl+S`. Quit with `Ctrl+Q`. Run `run_portable_check` again.
  Check the three variants again. Press `F2`. Hit pad `2` and press `F3`.
  Press `Ctrl+L` once to move its slice start right.
  Check that only this variant's linked pad changes. Pad `3` and pad `4` must keep their bounds and cuts.
  Press `Ctrl+S` before quitting.
- [ ] After quitting, check the external sample bytes:

  ```sh
  cmp "$check_root/samples/external.wav" "$check_root/external-before.wav"
  ```

  The command must exit 0.

### Physical-controller pass

- [ ] In the disposable run, open F12 and its MIDI tab.
  Learn separate buttons for GLOBAL `chop screen`, `save session` and `project picker`, and CHOP `slice start right`.
  Press Enter to save each assignment.
  Repeat the move/open/save and bound-edit checks with controller pads and these buttons.
  Keep picker navigation, typed names, sample lookup and folder moves on the keyboard or shell.
  This procedure does not claim controller-only project navigation.
- [ ] Report unavailable controls instead of marking their checks passed. Do not change the factory profile for this procedure.

### Listening pass and acceptance

The new local render is `renders/86d-portable-chops.wav`, stereo at 48 kHz, 9.216 seconds.
Each before/after part plays the same three restored slice pads. Short silences separate the hits.

| Time in seconds | Input and sample path | Comparison |
| --- | --- | --- |
| 0.000–1.152 / 1.152–2.304 | Keyboard, internal file | Before / after move |
| 2.304–3.456 / 3.456–4.608 | Keyboard, external file | Before / after move |
| 4.608–5.760 / 5.760–6.912 | Controller, internal file | Before / after move |
| 6.912–8.064 / 8.064–9.216 | Controller, external file | Before / after move |

- [ ] Listen to the render. Check each matching before/after part for the same tone, pitch, level and duration.
- [ ] In the disposable live run, listen to the assigned pads before and after the move. Check that no slice disappears or changes sound.
- [ ] Report live keyboard, physical-controller and listening results separately. All remain unverified until the owner reports them.

Archive: `openspec/changes/archive/2026-11-05-86d-portable-chops/`.
Change 87 and callback-budget corrections remain outside 86d. Push and PR need separate authorization.

## 87: Visual feedback and screen catalogue

Folder: `changes/archive/2026-11-07-87-status-badges`

The owner accepted 87 on 2026-11-07 and authorized archive, checks and squash merge in [PR #10](https://github.com/fl64/ptsr/pull/10).
Overall acceptance does not complete the unchecked keyboard, physical-controller, kitty or listening procedures below.
The archive applies all six deltas to the main specifications. Verification is recorded in its `tasks.md`, section 9.
Callback maximum-budget failures remain open. A successful p99 exit does not close that limit.
Closure order: owner acceptance, archive in the same PR, checks, then squash merge. Keep the branch after merge.

Current status correction: `KB` is permanent. Only the last used controller number remains beside it after highlight expiry or keyboard activity.
Input and save-state columns stay fixed. The permanent `*` is dim when saved and yellow when pending.
Read this marker in place of older `Unsaved` instructions. Keep the separate `Save failed` warning until saving succeeds.
Full port names stay in F12 Input. CPU and full xrun diagnostics stay in F12 Audio.
The unfinished SMC-PAD example is not a working profile. Use the archive's section 8 for the separate correction checks.

### Offline catalogue

Run from the repository root:

```sh
task screens
```

The command checks coverage and repeatable bytes. It writes 306 ANSI files under ignored `renders/screens/`.
It does not start the sampler, open devices, send kitty pictures or write projects and settings.
Each case has 80x26 and 120x40 versions. Use a terminal at least as large as the selected version.
View one file at a time:

```sh
cat renders/screens/pads-placements-80x26.ansi
```

| File in `renders/screens/` | Inspect |
| --- | --- |
| `pads-placements-80x26.ansi` | Combined file, copy and swap state, target roles and safety |
| `pads-edit-filter-80x26.ansi` | EDIT page, parameter names and shared frame |
| `pads-learn-80x26.ansi` | Persistent top status and fixed learning band |
| `pads-input-device-80x26.ansi` | Connected source labels and activity cue |
| `pads-messages-80x26.ansi` | Stable F1 frame, levels, origins and long history text |
| `chop-assign-order-80x26.ansi` | Whole Unicode target names inside the sidebar |
| `chop-128-slices-80x26.ansi` | Actual 128-slice fixture and bounded text |
| `fx-value-80x26.ansi` | Named master value and range panel on FX |
| `settings-question-profile-80x26.ansi` | Reset question and working keyboard answers |
| `settings-safety-80x26.ansi` | Save, sound, recording, xruns and release warnings together |
| `files-text-80x26.ansi` | Explicit text entry without a shifted frame |

- [ ] Inspect these files at both sizes. Replace `80x26` with `120x40` for the larger versions.
  Check whole graphemes, aligned frames, readable safety text and unchanged panel positions.
- [ ] Keep offline evidence separate. Fixture ports, meters and messages are synthetic data, not device observations.
  ANSI text does not prove kitty composition or sound.

### Prepare an isolated owner run

**Owner only.** The next command opens live audio and MIDI when the program starts.
An agent must not run it. Use your normal safe listening setup.
The command keeps this session's settings, state, samples and projects inside a new `target/` directory.
It does not copy or change your saved MIDI bindings.

```sh
task build
check_root="$(mktemp -d "$PWD/target/87-owner.XXXXXX")"
mkdir -p "$check_root/config/ptsr" "$check_root/data" "$check_root/state" \
  "$check_root/samples" "$check_root/projects"
cp renders/86d-portable-chops.wav "$check_root/samples/check.wav"
printf '[paths]\nsamples = "%s"\nprojects = "%s"\n[audio]\nbuffer_size = 1024\n' \
  "$check_root/samples" "$check_root/projects" > "$check_root/config/ptsr/config.toml"
printf 'Owner check folder: %s\n' "$check_root"
run_status_check() {
  XDG_CONFIG_HOME="$check_root/config" XDG_DATA_HOME="$check_root/data" \
    XDG_STATE_HOME="$check_root/state" ./target/release/ptsr
}
run_status_check
```

Use the same shell for later calls to `run_status_check`.
Press `Ctrl+P`. Type `87-screen`. Press `Enter`. Wait for the project to open.
Open F6. Select `check.wav` in the configured samples root. Press `Enter`. Hit pad `1`. Wait for loading.
Return to F2. Hit pad `1`. Press `Ctrl+C`. Hit pad `2` to place the copy. Press `Esc`.
For held-gate checks, copy a continuous sample at least 20 seconds long into this isolated samples folder.
Load that sample onto pad `3`. Do not use a short hit to claim a sustained release check.
If a device or release form is unavailable, record that limit instead of a pass.

### Keyboard checks

- [ ] Use `F2`, `F3`, `F4`, `F6` and `F12`. Check the same top status on all five screens.
  Inspect 120x40 and 80x26. Check alignment after each resize.
- [ ] On Pads, select pad `1`. Press `Ctrl+C`. Check COPY and the distinct empty and occupied target marks.
  Press `2`. Check ASK, target identity, safety and the shown keyboard answers. Cancel with the shown key.
  Press `Esc` again to drop the retained copy. Press `Ctrl+X`. Check SWAP and occupied-target `◎`.
  Hit pad `2`. Check a silent swap without a replacement question. Press `Ctrl+Z` to restore both pads.
- [ ] Press `M`. Check MARK and the marked-pad count. Hit loaded pad `1`.
  Check that marking still sounds. Press `Esc` to end marking.
- [ ] Open `F1`. Switch Help and Messages with `Tab` and `Shift+Tab`.
  Scroll Messages with `Up`, `Down`, `PageUp` and `PageDown`. Check a fixed frame and footer.
  Close help. Check that these navigation keys did not change the selected parameter underneath.
- [ ] Press `Ctrl+S`. Check the success cue and five visible seconds.
  Open F1 before it expires. Wait. Close F1. Check that hidden time did not consume its remaining visible time.
  Open Messages. Check one result with the original level and origin, not a duplicate on each frame.
- [ ] On Pads, change a value with `L`. Check the original pad, parameter, applied value and range.
  Change banks during its one-second lifetime. Check that the panel does not adopt the new pad's name.
  Open F1 after another value change. Wait two seconds. Close F1. Check that the expired panel stays absent.
- [ ] Check a numeric FX value and master volume. Check a categorical Settings choice.
  Check `next start` for Wave. Do not treat a configured buffer choice as a measured server buffer.
  Repeat an unchanged or limited value. Check that it does not renew the panel.
- [ ] Open `F10` on Pads, Chop, Files and FX. Check the bottom learning context, details and working controls.
  Check visible save and audio status above it. Check no false parameter-walk hint on Files.
  Cancel a pending or clear operation with the shown key. Check that no assignment changed.
- [ ] Open a pad name, file filter and project name. Type pad-block letters in each line.
  Check text entry without new keyboard pad sounds. Close each line with its normal Enter or Esc action.
  To repeat save-error checks, use section 86b with its own isolated preparation.
- [ ] Start an available recording with its existing keys. Check elapsed seconds during learning and F1 navigation.
  Stop it with the same recording action. Cancel the resulting placement if not needed.
  If recording is unavailable, check its refusal instead of inventing a running REC state.

### Controller checks

Use a connected controller's real profile and existing assignments. Do not install a new factory map for this check.
Record device name, preset and whether release uses NoteOff, zero-velocity NoteOn or a button release.
The default X-Session command buttons are not pad controls. A command-only device cannot complete the pad-hold check.

- [ ] Hit a pad or button. Check the activity cue on that source for one second. Turn an assigned knob.
  Check applied values and continuous-movement activity. Repeat its unchanged value. Check no false renewal.
- [ ] If two devices are available, alternate them. Check distinct source identities.
  Disconnect and reconnect one device. Check that the fresh connection does not inherit old activity.
  Report the two-device case as untested if only one device is available.
- [ ] Open keyboard text entry. Hit an existing controller pad. Check its normal MIDI route without typed characters.
  Do not expect controller pads to impersonate keyboard text. Check that releases still reach the held voice.
- [ ] Open an existing question. Use only controller answers shown for its actual bindings.
  Check the intended answer exactly once. Check that missing or disconnected bindings are absent from the answer row.
  If the profile has no answer binding, check keyboard fallback and report controller confirmation as unavailable.
- [ ] Enter learning. Check its target, pending change, refusal reason and actual bind or keep controls.
  Check that a clear request shows its current confirmation, not an earlier refusal.
  Cancel it. Check that old assignments still work.

### Kitty and graphics-off checks

These checks need the real terminal. Offline command-order tests cannot complete them.

- [ ] On F12 Audio, select Wave `picture`. Save with `Ctrl+S`. Quit with `Ctrl+Q`.
  Run `run_status_check` in a kitty-capable terminal. Load the source and open F1 over its waveform.
  Switch pages. Close F1. Check readable overlay text, then the current wave without stale overlays.
- [ ] Open a picker, question and value panel where applicable. Resize while each is open.
  Check the text it actually covers. Do not claim image removal for a panel that does not intersect the wave.
  Close each overlay. Check current data and positions without old-size pictures or lost nonintersecting waves.
- [ ] Select Wave `Braille` on F12 Audio. Save with `Ctrl+S`. Quit with `Ctrl+Q`.
  Run `run_status_check`. Repeat the overlay checks. Check readable Braille and text without kitty pictures.

### Listening and release checks

No new sound feature or WAV is part of 87. Generated ANSI files are not audio renders.
Use `renders/86d-portable-chops.wav` as the existing short sample if needed. It is not new listening evidence for 87.

- [ ] Play the continuous sample as a gate with a short release and no effect sends.
  Hold its keyboard key. Open and navigate F1. Release the key. Check prompt silence without a stuck voice.
  Repeat with the real controller release. Record keyboard and controller results separately.
- [ ] Play a known sample while changing values and navigating the UI. Check unchanged timing and no new clicks.
  Do not infer the callback maximum-budget result from this listening check.

Record each result with size, terminal, graphics mode and input source.
Leave unavailable hardware, kitty composition and listening cases unchecked.

## 87a: Numbered pad copy names

Folder: `changes/archive/2026-11-07-87a-copy-names`

The owner accepted 87a on 2026-11-07 after the short keyboard check.
The owner authorized archive, signed-off commits, push, a PR and squash merge. Keep the branch.
Copies use one `(copy N)` marker and the smallest free number across the current project's eight banks.
Manual markers use the same rule. The replacement target's current name remains occupied until placement succeeds.
Old names, samples, settings, sound, saved formats and controls stay unchanged.
The owner reported success for the first keyboard check: repeated copies, a copy of a copy, numbered names and silent placement.
Other keyboard checks, physical-controller checks and separate listening remain unverified. Their boxes stay open.

### Prepare an isolated owner run

**Owner only.** This procedure opens live audio and MIDI. The agent did not run it.
Use the main repository folder and your normal safe listening setup. No additional worktree is necessary.
These folders keep the test separate from your existing projects, scratch work and saved bindings.
Run the commands from the repository root. Keep this shell open.

```sh
task build
check_root="$(mktemp -d "$PWD/target/87a-owner.XXXXXX")"
mkdir -p "$check_root/config/ptsr" "$check_root/data" "$check_root/state" \
  "$check_root/samples" "$check_root/projects"
cp renders/83a-pad-copy.wav "$check_root/samples/check.wav"
printf '[paths]\nsamples = "%s"\nprojects = "%s"\n[audio]\nbuffer_size = 1024\n' \
  "$check_root/samples" "$check_root/projects" > "$check_root/config/ptsr/config.toml"
printf 'Owner check folder: %s\n' "$check_root"
run_copy_check() {
  XDG_CONFIG_HOME="$check_root/config" XDG_DATA_HOME="$check_root/data" \
    XDG_STATE_HOME="$check_root/state" ./target/release/ptsr
}
run_copy_check
```

Start in scratch. Open Files with `F6`. Select `check.wav`. Press `Enter`. Hit pad `1` and wait for loading.
Return to Pads with `F2`. Press `Ctrl+N`. Replace the name with `kick`. Press `Enter`.
Keep pad edit and text input closed for copying. The keys below are defaults. Use your replacements if necessary.

### Keyboard pass

- [x] Hit pad `1`. Press `Ctrl+C`. Hit pad `2`. Check `kick (copy 1)` and an unchanged source.
  Copy pad `1` to pad `3`. Check `kick (copy 2)`. Copy pad `2` to pad `4`.
  Check `kick (copy 3)`, without another nested marker. Every placement hit must stay silent.
- [ ] Hit pad `3`. Press `Backspace`. Copy pad `4` to pad `3`.
  Check reuse of `kick (copy 2)`. Press `Ctrl+Z`. Check the empty target.
  Repeat the copy. Check the same available number, not a new counter value.
- [ ] Clear pads `3` and `4` with `Backspace`. Keep only `kick` and `kick (copy 1)`.
  Copy pad `1` onto pad `2`. Check the proposed `kick (copy 2)` before confirmation.
  Press `Esc`. Check the unchanged target and retained COPY mode. Hit pad `3`.
  Check `kick (copy 2)`. Press `Ctrl+Z`. Check that only pad `3` becomes empty.
  Repeat replacement of pad `2`. Press `Enter`. Check its proposed name. Undo and check `kick (copy 1)` again.
- [ ] Copy pad `2`. Press `Ctrl+2`. Hit empty pad `1` in bank 2.
  Check `kick (copy 2)`: the name in bank 1 reserves number 1.
  Press `Ctrl+S`. Record both banks' names. Quit with `Ctrl+Q`. Run `run_copy_check` again.
  Check those names and settings. Copy the bank-2 pad to an empty target. Check `kick (copy 3)`.
- [ ] Create a disposable project with `Ctrl+P`. Load `check.wav` onto its pad `1`.
  Rename it to `kick (copy 7)`. Copy it to pad `2`. Check `kick (copy 1)` and the unchanged manual source name.
  The names in scratch must not reserve numbers in this project.
  Save and restart. Copy pad `2` to pad `3`. Check `kick (copy 2)`.
- [ ] Rename a loaded source to `rec 2 2 2`. Copy it to an empty target.
  Check `rec 2 2 2 (copy 1)` and the unchanged source text.
  Repeat with a long Unicode name. Check the full text in the rename line, not only its clipped pad cell.
  Repeat with `kick (copy 01)`. Check the literal base: `kick (copy 01) (copy 1)`.

### Physical-controller pass

Use an actual device's existing pad and command assignments. Record its name and preset.
Use notes 36–51 from a device without a profile, or existing learned pad buttons.
The X-Session Pro's default command buttons are not a pad grid. The SMC-PAD example is not a working profile.
Use keyboard commands where no controller binding exists. Record that limit instead of claiming controller-only operation.
Do not reset a factory map or overwrite performance bindings for this check.

- [ ] Repeat the repeated-copy, copy-of-copy and cross-bank checks with physical pad targets.
  Use existing controller copy and bank commands where available. Check the same names and silent placement.
- [ ] Repeat the occupied-target question, cancellation and undo.
  Use only confirmation controls shown for the actual bindings. Extra pad hits must not answer the question.
- [ ] Save and restart the disposable project. Repeat a copy with controller targets.
  Check the next free number. Record unavailable notes, learned buttons and command bindings separately.

### Listening and automated evidence

- [ ] Listen to local `renders/83a-pad-copy.wav`. The first voice is the source and the second is its copy.
  Check the same sound, without a new click or extra placement hit.
- [ ] Play the source and copy in the disposable live run. Check the same settings and sound after restart.
  Report keyboard, physical-controller and listening results separately.

No new sound feature or render is part of 87a. The existing WAV remains byte-identical before and after implementation.
Its SHA-256 is `2a8e1b1647d7ac6e493c2d00dd319093823d09f1b3785b49924e475c69cc3bfd`.
`task check` passed 1117 workspace tests, including 780 App tests. Fifteen new tests and existing copy tests cover all 17 delta scenarios.
`task build` passed. `task openspec:check` reported `Totals: 16 passed, 0 failed (16 items)`.
`task screens` passed four tests and generated 306 ANSI files at 120x40 and 80x26.
These commands did not open live audio or MIDI. They do not establish any unchecked owner result above.
These implementation results precede archive. Refer to [the archived tasks](../openspec/changes/archive/2026-11-07-87a-copy-names/tasks.md#owner-acceptance-and-closure-2026-11-07) for closure evidence.
Earlier callback maximum-budget failures remain open. Owner acceptance does not close that separate issue.

## 95: Transport and metronome

Folder: `changes/archive/2026-11-08-95-transport`

The owner accepted 95 and authorized archive and checks on 2026-11-08.
Overall acceptance does not establish individual keyboard, physical-controller or listening results. Their checkboxes remain open.
The agent did not open live audio or MIDI. This procedure is for the owner.
Change 96 adds saved A1 playback to this transport. Space keeps preview playback on Chop and Files.
The current build saves format 5. Section 96 gives its backup and trial procedure. Keep existing projects outside this isolated run.

### Prepare an isolated owner run

Run these commands from the repository root. Keep this shell open.

```sh
task build
check_root="$(mktemp -d "$PWD/target/95-owner.XXXXXX")"
mkdir -p "$check_root/config/ptsr" "$check_root/data" "$check_root/state" \
  "$check_root/samples" "$check_root/projects"
cp renders/83a-pad-copy.wav "$check_root/samples/check.wav"
printf '[paths]\nsamples = "%s"\nprojects = "%s"\n[audio]\nbuffer_size = 1024\n' \
  "$check_root/samples" "$check_root/projects" > "$check_root/config/ptsr/config.toml"
printf 'Owner check folder: %s\n' "$check_root"
run_transport_check() {
  XDG_CONFIG_HOME="$check_root/config" XDG_DATA_HOME="$check_root/data" \
    XDG_STATE_HOME="$check_root/state" ./target/release/ptsr
}
run_transport_check
```

### Keyboard pass

- [ ] On Pads, check stopped transport at 120.0 BPM. Press `Alt+M`, then `Space`.
  Check the play indicator, metronome indicator and bar/beat. Hold Space. Check that it does not toggle repeatedly.
- [ ] Press `Ctrl+Up` and `Ctrl+Down`. Check 1 BPM steps. Add Shift. Check 0.1 BPM steps and the value popup.
  Repeat with the Russian layout. Record the terminal and whether modified arrows reach the program.
- [ ] Switch between Pads, FX and Settings. Check the same clock. Press Space to stop.
  Check bar 1, beat 1 after restarting. On Chop and Files, check that Space still controls preview.
- [ ] Open Files. Load `check.wav` onto pad `1`. Return to Pads. Start transport, then hit pad `1`.
  Stop transport before the sample ends. Check that the pad tail continues.
- [ ] Start transport with the metronome enabled. Disable it for several beats, then enable it between beats.
  Check that the clock continues and the next click follows its beat without replaying missed clicks.
- [ ] Use `Ctrl+P` to create project `A`. Set 172.3 BPM and enable the metronome. Save with `Ctrl+S`.
  Create project `B`. Set 90 BPM and disable the metronome. Save it.
  Start transport. Switch to `A`. Check 172.3 BPM, metronome enabled and stopped transport.
  Switch to `B`. Check its separate settings. Select `B` while its clock runs. Check uninterrupted position.
- [ ] Save. Quit with `Ctrl+Q`. Run `run_transport_check` again.
  Check the last project's tempo and metronome preference, with transport stopped.
- [ ] Resize to 80x26 and 120x40. Open F1 and a replacement question while transport runs.
  Check readable clock, mode and question labels. Check that learn selection does not start transport or change tempo.

### Physical-controller pass

Do not reset existing profiles or performance bindings. Use available spare controls in this isolated config.
The SMC-PAD example remains an incomplete template. Do not infer controller-only reach from these steps.

- [ ] In F12 MIDI, open GLOBAL. Learn separate transport and metronome buttons. Press Enter to save each assignment.
  Check one action per press, without another action on release. Repeat coarse and fine tempo buttons.
- [ ] Learn the GLOBAL tempo knob. In jump mode, check both limits: 40 and 300 BPM.
  Check the BPM popup. Save and restart. Check the same assignments and unchanged existing controller roles.
- [ ] With two available controllers, check pickup separately. Change tempo with the keyboard between knob movements.
  Check that one controller does not unlock the other. Check scale response separately.

### Listening and offline evidence

- [ ] Listen to local `renders/95-transport.wav`. Hear 120 BPM, then 90 BPM, a stop, then a restart at 172 BPM.
  Check the stronger first beat and quieter beats 2–4. Check silence during the stop and after the final tail.
- [ ] In the isolated live run, check metronome timing and pad tails by ear.
  Record listening and physical-controller results separately. Leave unavailable checks open.

`task test` generates the WAV without a sound card. The file is local, not an attached recording of live hardware.
The callback benchmark includes 32 voices, effects, transport and metronome at 172 BPM.
Keep its existing budget limits. The p99 exit checks pass. Earlier maximum-target failures remain unresolved.
See [verification limits](../openspec/changes/archive/2026-11-08-95-transport/tasks.md#implementation-verification). A later passing run does not close maximum-budget or slow-core investigations.

## 96: One looped pattern and slice Pitch Gate

Folder: `changes/96-pattern-play`

The copied 96 trial binary plays one saved A1 pattern across all eight banks. That older binary does not record patterns.
Use the current [97 recording trial](#97-record-and-erase-a1) for recording. A step editor remains planned.
The temporary file-entry route requires a closed program. There is no file watcher or slot selector.
Owner keyboard, physical-controller and listening acceptance remain separate from automated checks.
Earlier callback maximum-target failures remain open, even when a command's p99 gate passes.

### Ready local owner trial

**Owner only.** This command opens normal audio and MIDI. The agent must not run it without device isolation.
The prepared copy is local and ignored by Git. It is not a repository sample or an attached recording.
Run this command from the repository root:

```sh
./target/96-owner-trial/start.sh
```

Save and close any existing ptsr instance before this trial. Do not run both copies for the comparison.
The project `96-slices` opens stopped, in bank 1, with a two-bar A1 at 92 BPM.
It contains eight copied slice pads from an owner project and one copied recording.
External downloads and unrelated sample assignments are absent. The retained older chop states stay with the copied recording.
Master starts at -12 dB. Metronome starts off. Pad `W` has pitch +12 and the name `PG +12`.
These are trial-only settings. The original project and recording remain unchanged.
The start script uses a copied binary and separate config, scratch, project, sample and log folders.
It checks that startup paths remain inside the trial folder. It does not copy saved owner MIDI bindings.
Do not change the trial's roots or load external files for these checks.

`start.sh --check` checks paths without starting sound or MIDI. It does not establish a live-device or listening result.
If the local trial is absent, prepare a disposable project with the developer procedure below.
Do not substitute an existing owner project for a missing trial.

### Keyboard pass

Use the prepared trial. Start at a comfortable listening level. The keys below are defaults.

- [ ] Check `96-slices`, stopped transport and A1 at bar 1, beat 1.
  Press `Space` on Pads. Check the two-bar rhythm and repeating A1 position.
  Check that the absolute transport continues while A1 returns to bar 1.
- [ ] Leave a different pad selected while the pattern plays. Change banks with `Ctrl+2`, then `Ctrl+1`.
  Check that notes retain their sounds and absolute addresses. Playback must not select pads or simulate held input.
  Check PLAY, brief hit flashes and the existing keyboard/controller identity.
- [ ] Press `Ctrl+Up`, then `Ctrl+Down`. Check one-BPM changes.
  Add Shift for 0.1 BPM. Check the same beat for pattern and metronome after enabling it with `Alt+M`.
  Disable the metronome again before comparing tails.
- [ ] Hit `W`. Check `PG +12` and its linked slice.
  The 96 trial copy shows `PG on`. A binary with the 97 feedback clarification shows `PG A1`.
  Press `Alt+G`. Check `PG off` and the value popup. Compare subsequent automatic tails.
  Press it again. Check the return to On. The cut uses the next grid line, including an empty step.
- [ ] Hit `W` manually with On, then Off. Check unchanged manual sound without an imposed step cut.
  For solo preview, stop transport on Pads before opening its slice on `F3`.
  Check that preview remains independent of Pitch Gate. Return with `F2` and restart the pattern if necessary.
  Do not infer a whole-file or zero-pitch cut from the saved On label.
- [ ] On Pads, open edit with `Enter`. Toggle Pitch Gate again.
  Check the same mode-row label without a new page or changed knob targets. Close edit with `Esc`.
  Open `F1`. Check effective keys and the explanation of positive-pitch linked slices. Close help with `F1`.
- [ ] Keep `W` selected. Press `Ctrl+X`. Hit `E` as the swap target.
  Check the moved complete pad state and the same future rhythm with its sounds.
  The existing short swap fade can affect a sounding voice. No delayed or doubled hit should follow.
  Press `Ctrl+Z`. Check that both pads and all hit addresses return together.
- [ ] Press `Space` on Pads to stop. Check stopped transport and loop position at bar 1, beat 1.
  Ordinary one-shot and shared-effect tails can finish. Gate and Pitch Gate notes use their required releases.
  Press `Space` again. Check one first hit, without an old release cutting a new note.
- [ ] Open `F4` and `F12` separately. Check the same accepted A1 clock. Space controls transport there.
  On Chop and Files, Space keeps its preview role. Return to Pads before the next transport command.
- [ ] Stop. Press `Ctrl+S`. Check the save result. Quit with `Ctrl+Q`.
  Run the same start script. Check retained tempo, A1, pad state and Pitch Gate, with playback stopped at the beginning.

The Chop interaction remains an open specification question before archive.
The local trial fades current pad voices on entry, but future pattern hits continue on Chop while transport runs.
Space previews there instead of stopping transport. Use Pads to stop first when solo listening is necessary.
This observed behavior is not listening acceptance or an agreed future suppression policy.

### Physical-controller pass

Use only available controls in the isolated trial. Do not reset a factory map or replace performance bindings.
The SMC-PAD example is not a verified profile. Controller layers, holds and physical comfort remain work for their own changes.
Use keyboard fallback where a binding is absent. Record that dependence rather than claiming controller-only reach.

- [ ] Open F12 MIDI. Learn an unused button for GLOBAL transport toggle.
  Press Enter to save it. Learn another button for PADS Pitch Gate toggle.
  A knob is not a button. Check pending confirmation and cancellation before retaining a binding.
- [ ] On Pads, press the transport button. Check one start. Release it. Check no second toggle.
  Select `W` with a physical pad if available. Press the Pitch Gate button.
  Check its target, mode-row state, applied-value popup and future sequenced tail.
- [ ] Repeat with pad edit open and closed. Check the same intended PADS binding.
  Save and restart. Check retained trial bindings, without changing the owner's saved map outside the trial.
- [ ] Use existing learned swap and undo commands for the keyboard swap check.
  A swap needs its command and target hit. An extra bank change is necessary only for a cross-bank target.
  Do not count keyboard fallback as an implemented SMC-PAD control.
- [ ] Compare a manual physical pad hit with its equivalent keyboard hit at matching velocity.
  Pitch Gate must not impose an automatic step cut on either manual hit.
  Record the device, preset, message form and unavailable controls.

With a selected target, a learned Pitch Gate button needs one press and no page change.
Keyboard Alt+G uses one modifier chord. F10 and F12 are setup routes, not repeated performance steps.
No physical action-count result establishes comfort on an unmeasured SMC-PAD.

### Closed-program entry and backup procedure

This route is for developer review or an owner who explicitly wants to enter raw events.
The ready local trial above does not require it.

1. Use a disposable project with two loaded pads. Save it with `Ctrl+S`.
2. Quit with `Ctrl+Q`. Keep the program closed during every file edit.
3. Copy the complete project folder to a separate backup folder before the first format-5 save on valuable work.
4. Open only the disposable `project.toml`. Keep its existing version, tempo, pads, chops, paths and effect tables.
5. Add `pattern = "A1"` before the first table header. Do not add a second version or tempo field.
6. Append the pattern table below. Do not append a second A1 entry to a file that already has one.
7. Reopen the same project. Check the stopped A1 label. Follow the playback, swap, undo and save checks above.

```toml
[[patterns]]
slot = "A1"
bars = 2
quantize = "1/16"
strength = 100
hits = [[0, 0, 100, 0], [1920, 1, 110, 0], [3840, 0, 100, 0], [5760, 1, 110, 0]]
```

Each hit is `[raw tick, absolute pad, velocity, gate ticks]`. A quarter note has 960 ticks. A bar has 3840 ticks.
Pad 0 is bank 1 key `1`. Pad 1 is bank 1 key `2`. A one-shot ignores gate ticks. A zero-length Gate note is silent.
To replace an existing pattern, edit its one table instead of appending another.
[The format contract](../openspec/changes/96-pattern-play/design.md#saved-format) gives grids, ranges and hidden-hit rules.
Quantize changes playback, not saved raw timing. Off uses a 1/16 boundary for Pitch Gate.

An older binary refuses format 5. To return to it, close ptsr and restore the complete pre-upgrade backup.
Do not lower only the version number or remove new fields to force a downgrade.
The local trial's original source is unchanged, but it is not a universal backup of the owner's other work.
Never edit a protected, invalid or newer file while ptsr keeps that session open.

### Independent device-free workflow review

A reviewer must keep the owner's current sound and MIDI inaccessible.
A nonexistent JACK server alone does not isolate MIDI. Keep the forced JACK backend, 1024-frame setting and isolation variables from AGENTS.md.
The local review helper `target/96-implementation/isolate_terminal.py` additionally restricts device files, network connections and writes through kernel Landlock.
Its preflight must pass before any terminal start. Failure means no start, not a fallback to ordinary ptsr.
Container namespace attempts on this host failed, so their commands provide no isolation evidence.

Check the owner's graph without changing it. If another ptsr node exists, do not start a second live sampler.
Use recorded production keyboard/MIDI dispatch, offline callbacks and saved-file round trips for the workflow instead.
Report the interactive terminal pass as blocked. Keep simulated input, terminal observations, hardware and listening results distinct.
Do not stop or reconfigure the owner's existing process to finish a review.

### Listening and automated evidence

Regenerate the required generated-sound WAVs without opening a device:

```sh
cargo test -p engine --test pattern_renders -- --nocapture
```

| Local file | Time in seconds | Listen for |
| --- | --- | --- |
| `renders/96-pattern-play.wav` | 0–8 | Two two-bar loops at 120 BPM with short percussion and released gate bass |
| Same file | 8–11.2 | Third loop at 150 BPM without drift or a doubled first hit |
| Same file | 11.2–14.4 | Fourth loop with later snare events after accepted pattern replacement |
| Same file | 14.4–14.9 | Intentional silence |
| `renders/96-pitch-gate.wav` | 0–2.5 | Pitch Gate On: first cut at 0.125, a later note and the loop seam at 2 seconds |
| Same file | 2.5–5 | Matching pattern with Off, allowing the tone to continue |
| Same file | 5–7.5 | Manual hits with On, matching the Off control |

- [ ] Listen to both files. Check stable timing, intended short tails and unchanged attacks.
- [ ] Compare On, Off and manual sections. Check the cut at an empty step and the seam without an unintended extra attack.
- [ ] Listen to the copied owner-slice trial. Report unwanted clicks with the pad, Pitch Gate state, exact action and time.
- [ ] Report keyboard, physical-controller and listening results separately. Leave unavailable checks open.

Both files are local, not recordings of live hardware. Their tests check finite samples, no clipping, levels, silence and timing.
Manual references also check both channels within 0.000001 per sample and the existing short-cut envelope.
The separate dense benchmark uses 4096 events, 32 voices, shaping, both effects and the 172 BPM clock.
Read maxima and counts above 50%, not only command success. Earlier maximum-budget and slow-core failures remain open.
[The active task record](../openspec/changes/96-pattern-play/tasks.md#required-listening-files) keeps literal commands and measured limits.

## 97: Pitch Gate setting and applicability

This feedback boundary changes no sound. The [recording trial below](#97-record-and-erase-a1) covers the later completion of the active change.
Use the newly built `target/release/ptsr` in a disposable project, not an owner project.
The 96 trial script uses a copied binary. Do not assume that copy includes the new feedback.
Keep keyboard, physical-controller, live-terminal and listening results separate.
Automated tests use recorded input and offline audio. They do not establish hardware or listening acceptance.

### Keyboard feedback check

1. Open Pads with F2.
2. Select a linked slice pad with a manual hit.
3. Open edit with Enter.
4. Select TONE with Tab or Shift+Tab.
5. Select Pitch with I or K.
6. Use J or L to set Pitch above zero.
7. Press Alt+G until the mode row shows `PG A1`.
8. Check `PG on: A1 only, manual unchanged` in the last help row.
9. Hit the pad manually.
10. Check unchanged manual sound and the same A1-only explanation.
11. Press Alt+G once.
12. Check `PG off` and the popup's target pad.
13. Press Alt+G again.
14. Check `PG on: A1 only` in the popup.
15. Set Pitch to zero, then below zero, with J/L.
16. Check `PG n/a` and `needs Pitch > 0` without another toggle.
17. Select a whole-file pad, a detached piece and an empty pad in turn.
18. Keep Pitch Gate On for each pad.
19. Check `PG n/a` and `needs linked slice`, including at zero pitch.
20. Return to the linked slice with positive pitch.
21. Check `PG A1` without changing the saved switch.
22. Open F1.
23. Check A1-only scope, positive pitch, linked slices and the next-grid cut, including empty steps.
24. Check unchanged manual hits and previews, plus the effective toggle key.
25. Close F1.

Use the effective owner key instead of Alt+G if its binding differs.
Repeat with edit closed, each edit page, hidden pads and both 80x26 and 120x40 terminals.
Check the unchanged mode, scope and parameter-row positions within each configuration.
Check the pad identity in every popup. No new top badge or parameter page must appear.
The first two Pads help rows contain compact commands. F1 retains commands that no longer fit there.
Learning keeps its existing help priority. Save and audio warnings keep their existing positions and priority.
Do not create a real save or audio failure for this check. Offline fixtures cover those combinations.

With a prepared A1, compare stopped transport, playback and simultaneous manual hits.
The label describes the saved setting and applicability, not a report that the current sound has a cut.
An absent A1 also keeps `PG A1` for an eligible On setting.
Manual hits remain unchanged in all three cases. Short slices need not produce an audible On/Off difference.

### Learned-controller feedback check

1. Use the disposable project's existing learned Pitch Gate button.
2. Select the same pad with a physical pad if available.
3. Press the learned button once.
4. Check one saved-state change and the popup's target pad.
5. Check the same source or pitch explanation as the keyboard path.
6. Repeat with edit open and closed.
7. Open F1.
8. Check the actual learned button and effective keyboard key.

If no button exists, follow the F12 learning procedure in the 96 controller check.
Do not reset a device profile or replace the owner's saved performance bindings.
Raw note and CC tests do not establish physical SMC-PAD support or comfort.

### Sound and local screen evidence

No new sound render is necessary for this feedback-only change.
Use `renders/96-pitch-gate.wav`: 0–2.5 seconds is On, 2.5–5 seconds is Off, and 5–7.5 seconds is manual On.
Listen for the existing automatic cut and unchanged manual section. Do not expect a pitch change.
Owner listening, physical-controller use and a live terminal check remain unverified.

`task screens` generates the local ANSI frames without devices or owner-project writes.
The base catalogue includes `pads-pitch-gate-{off,a1,source,pitch}-{80x26,120x40}.ansi` in `renders/screens/`.
Additional frames cross those states with every edit page, hidden pads and safety warnings.
For example, compare `pads-pitch-gate-source-hidden-TONE-safety-80x26.ansi` with its `a1` variant.
ANSI frames are not evidence of kitty image composition or live-device behavior.

## 97: Record and erase A1

Folder: `changes/97-pattern-record`

Use the current `target/release/ptsr`, not the copied 96 trial binary.
These procedures describe the implemented software paths. They do not claim completed owner, hardware or listening acceptance.
Leave every untested box open. Keep callback maximum acceptance separate from a successful p99 result.
Do not use a valuable project for this trial. No project-file edit is necessary.

### Prepare a disposable trial

**Owner only.** The final command opens normal audio and MIDI. It does not isolate devices.
Save and close another ptsr instance before this trial. The agent must not execute the final command without separate device isolation.
The preparation uses Python's standard library and generated sounds. It leaves the owner's saved files unchanged.
Run this block from the repository root after `task build` succeeds:

```sh
trial="$(mktemp -d "$PWD/target/97-owner-XXXXXX")"
python3 - "$trial" <<'PY'
import json
import math
from pathlib import Path
import struct
import sys
import wave

root = Path(sys.argv[1]).resolve()
for name in ("config/ptsr", "data", "state", "projects", "samples"):
    (root / name).mkdir(parents=True, exist_ok=True)
config = "[paths]\nprojects = " + json.dumps(str(root / "projects"))
config += "\nsamples = " + json.dumps(str(root / "samples"))
config += '\n[audio]\nbackend = "auto"\nbuffer_size = 1024\n'
(root / "config/ptsr/config.toml").write_text(config)
for name, frequency, count in (("low", 55.0, 2400), ("high", 110.0, 2400),
                                ("gate", 165.0, 192000)):
    frames = []
    for i in range(count):
        envelope = (0.1 * math.sin(math.pi * i / (count - 1)) ** 2 if count == 2400
                    else 0.1 * min(1.0, i / 480, (count - 1 - i) / 480))
        samples = [round(32767 * envelope * math.sin(2 * math.pi * frequency * ratio * i / 48000))
                   for ratio in (1.0, 1.5)]
        frames.append(struct.pack("<hh", *samples))
    with wave.open(str(root / "samples" / (name + ".wav")), "wb") as output:
        output.setparams((2, 2, 48000, 0, "NONE", "not compressed"))
        output.writeframes(b"".join(frames))
print("Disposable trial:", root)
PY
XDG_CONFIG_HOME="$trial/config" XDG_DATA_HOME="$trial/data" \
XDG_STATE_HOME="$trial/state" ./target/release/ptsr
```

The generated sounds have different left and right frequencies. A plain hit is quieter than a Shift hit.
The files use no third-party recording. Keep the printed trial path for the restart check.
The trial has separate config, session, project, sample and log folders. Factory profiles still come from this repository.
The setup does not copy the owner's learned controls. It does not add a factory SMC-PAD assignment.

### Keyboard pass

Use the default keys below only if F1 shows them. Use the effective key if an owner override replaces it.

- [ ] Open F12 Paths. Check that both roots are inside the printed trial folder.
  Do not continue if either root points to an owner folder.
- [ ] Press Ctrl+P. Type `97-record`. Press Enter to create the disposable project.
  Open F6. Load `low.wav` on pad `1`, then `high.wav` on pad `2`, with Enter and the target pad.
  Load `gate.wav` on pad `3`. This four-second tone makes held-release checks audible.
  Open F2. Select pad 3 with the arrows. Use H to set its mode to Gate.
- [ ] Open F2. Press Ctrl+R while stopped. Check COUNT 4, 3, 2, 1, then REC A1.
  End this empty pass with Ctrl+R. Stop with Space. Start another count-in with Ctrl+R.
  Hold pad 3 and press Esc. Check stopped transport, no recorded warm-up row and a working original release.
- [ ] Start again. After REC A1 appears, play a two-bar rhythm with pads 1 and 2.
  Include plain and Shift hits. Press Ctrl+R once to end the pass.
  Check continuing playback, one quantized repeat and no extra hit immediately after a manual hit.
- [ ] Press Ctrl+R during playback. Add a new high hit on another beat. End with Ctrl+R.
  Check the original clock, old rhythm and new hit. Press Alt+Z once.
  Check that only the latest pass disappears. Record it again for the next steps.
- [ ] Open Alt+N. Change Quantize among all grids. Check Strength at 0, 50 and 100.
  Check the later repeats, not a rewritten raw take. Close with Enter, Esc or Alt+N.
  With recording Idle, change Bars at both limits, then return to two bars.
- [ ] Start recording during playback. Try Bars. Check an instruction to finish the pass first.
  End with Ctrl+R. Stop with Space on Pads. Start with Ctrl+R.
  While COUNT is visible, try Quantize and Strength. Check a refusal that names count-in.
  Close controls. End or cancel with Ctrl+R. If transport stopped, start playback with Space on Pads.
  Do not create a real audio or save failure. Offline tests cover Finishing and recovery pressure.
- [ ] Start recording during playback. Press Alt+E. Hold the low pad across one of its notes.
  Check ERASE A1, the accepted absolute target and silence for that erased note.
  Release before another note. Check that the release excludes its boundary.
- [ ] Press Alt+E again. Add a new low hit. Change the grid and strength during the same pass.
  End the pass. Press Alt+Z once. Check restoration of the old rhythm and settings together.
  Start and end an empty pass. Check the unchanged restored rhythm.
  Alt+Z must then undo the earlier overdub, not an empty-pass step.
- [ ] Make a pad-volume edit after a completed pass. Press Alt+Z.
  Check a refusal to cross the newer edit. Use normal Undo on Pads first.
  Do not accept a pattern Undo that changes that pad volume.
- [ ] On Pads, select pad 3 with the arrows. Press H until its mode is Gate. Hold pad 3.
  Change bank or screen before release. Check that release still ends the original pad.
  Repeat across record end and erasure toggles. With no key releases, check the stated one-loop erasure fallback instead of a hold.
- [ ] Repeat recording controls on F6, F3, F4 and F12. Keep earlier notes playing.
  On Chop, check actual pads and linked slice selection during pattern work.
  Check that Space and cursor preview do not enter A1. Stop on Pads before solo preview outside pattern work.
- [ ] Open a name or filter line, picker, question or learning mode. Try the recording commands.
  Check the original input owner and no underlying record, erase or controls toggle.
  Check that an earlier permitted pad release still works.
  Close each blocking mode through its normal action before the next check.
- [ ] Open F1 while recording. Check the live phase on Keys and Messages.
  Page Keys with Alt+PageDown and Alt+PageUp. Check the final rows and the last-page clamp.
  Change terminal size between 80x26 and 120x40. Check readable controls and unchanged save/audio status positions.
  Close F1 with Esc.
- [ ] Repeat the command keys in Russian and with pad edit open. Check effective owner overrides and `none` where applicable.
  Holding a recording, erasure or controls key must not toggle repeatedly. Releasing it must not toggle.
- [ ] Press Ctrl+S during a pass with a Gate held. Check pass completion before the save result and continuing playback.
  Close with Ctrl+Q. Restart with the same XDG variables and trial path.
  Check the saved take and settings, stopped transport and clock zero. Earlier physical holds must not return.

Alt+R remains microphone recording. It is a hold with key releases and a toggle without them.
Ctrl+T remains output recording. A conflicting recording target must refuse without stopping the active target.
Input gain, noise removal, a step editor and wider Pitch Gate sound behavior are not part of this trial.

### Learned-controller pass

Use only unused controls in the disposable trial. Do not replace the owner's saved performance map.
Keep unavailable hardware checks open. Raw note and CC tests do not establish physical SMC-PAD support or comfort.

1. Open F12. Use Tab to select MIDI.
2. Select GLOBAL with the arrows. Press Enter to open it.
3. Select `record pattern`. Press Enter to start learning.
4. Press the unused controller button. Check its pending assignment.
5. Press Enter to accept that assignment.
6. Repeat steps 3–5 for the other button labels below.
7. Repeat the first-take, overdub, erasure, settings and Undo steps with those buttons and physical pads.
8. Check that release does not toggle a command again.
9. Repeat on every current screen and across bank changes with a pad held.
10. Repeat blocked-input and F1 paging checks.
11. Save the trial. Restart with the same XDG variables.
12. Check retained trial assignments and stopped transport.

| F12 label | Physical control |
| --- | --- |
| `record pattern` | Button |
| `pattern controls` | Button |
| `erase pattern under held pads` | Button |
| `undo pattern pass` | Button |
| `previous pattern parameter` | Button |
| `next pattern parameter` | Button |
| `decrease pattern value` | Button |
| `increase pattern value` | Button |
| `previous help page` | Optional button |
| `next help page` | Optional button |
| `pattern bars` | Optional absolute knob |
| `pattern quantize` | Optional absolute knob |
| `pattern strength` | Optional absolute knob |

If absolute knobs are available, learn the three knob labels above.
Check jump, pickup and scale separately. Check each device and knob's own pickup state.
A refusal during count-in or finishing must not show a changed value or create A1 accidentally.
Record the device, preset, actual messages and keyboard fallback. Do not infer factory assignments from learned test routes.

### Listening file

Regenerate the file without opening a device:

```sh
cargo test -p ptsr --bin ptsr complete_recording_workflow -- --nocapture
```

Listen to `renders/97-pattern-record.wav`. It is a local generated-sound render, not attached live hardware audio.
Count-in precedes the captured file. The file starts at accepted A1 tick zero and lasts 28 seconds.

| Time in seconds | Listen for |
| --- | --- |
| 0–4 | Manual low hit at 0.105 and high hit at 0.750 |
| 4–8 | Quantized low hit at 4.125 and high hit at 4.750, without an extra raw repeat |
| 8–16 | Existing rhythm plus a new manual high hit at 8.500 and its repeat at 12.500 |
| 16–20 | No old low hit at 16.125, then the new manual low hit at approximately 16.667 |
| 20–21 | New high hit at 20.500, new low at approximately 20.708 and old high at 20.750 |
| 21–24 | One pass Undo restores the previous pattern while the clock continues |
| 24–28 | Original low returns at 24.125, high hits at 24.500 and 24.750, then silence after 24.800 |

- [ ] Listen to both channels. Check clear attacks, intended silences and no unintended clicks or doubled hits.
- [ ] Compare the last restored loop with the rhythm before erasure. Check that the overdub remains.
- [ ] Report keyboard, physical controller, terminal reporting and listening results separately.

The test checks every sample against an independent manual reference in both channels within 0.000001.
It checks 19 onsets within one frame, finite values, no clipping and the required silence windows.
Each onset's RMS matches the independent reference within 0.01 dB. Adjacent-sample changes stay within the generated reference envelope.
These automated checks do not prove listening quality, physical latency, kitty image composition or terminal key reporting.

### Callback acceptance

Run `task bench` separately from live audio. Do not run another build or test workload during a timing comparison.
Read the separate active recording and erasure groups, their release and Finishing boundaries, and their maximum/count lines.
The new groups fail for any measured wall or thread CPU maximum above 50 percent.
Original playback and switching checks retain their existing exit rules.
Keep every historical overrun visible. A later passing maximum or p99 does not resolve an earlier missed target.

## 96-98h: Patterns and edit frames

Folders: `changes/archive/2026-11-12-96-pattern-play` to `changes/archive/2026-11-12-98h-empty-pad-tone`

Use a disposable project or a copy. Run `task build`, then start `target/release/ptsr`.
These steps describe the implemented software paths. Leave every untested box open.

### Record and quantize

- [ ] Open Alt+N. Set quantize 1/4. Press Esc. Press Ctrl+R, wait for the count-in and play a kick on two beats. Press Ctrl+R.
- [ ] Set quantize 1/16 in Alt+N. Check that the kicks stay on their beats.
- [ ] Record a snare between beats. Check that it lands on the nearest 1/16 cell.
- [ ] Hit one pad twice in one 1/16 cell in one pass. Check that one hit stays.

### The A1 view

- [ ] Start the program. Check that the A1 rows show at once, under the splash at 120x40.
- [ ] Check the header: `view 1/16 · quantize 1/4 100 % · bank N`, and the pad row label `pad Q`.

### Erase and clear

- [ ] Outside a pass, press Alt+E on F2. Check `ERASE` and `N hits` on the pads.
- [ ] Press a pad. Check that it makes no sound and its hits go. Press Ctrl+Z, then Esc.
- [ ] During playback, press Alt+E and a pad. Check that it stops in the next loop without a click.
- [ ] In Alt+N, select clear A1 and press Enter. Answer keep with Esc, then repeat and answer yes. Press Ctrl+Z.

### Edit frames

- [ ] Alt+N: check the `EDIT · PATTERN CLEAR` frame and that H and Backspace do not change the pad.
- [ ] F4: check one frame `EDIT · FX 1 FX 2`. Press Tab: FX 2 is lit.
- [ ] F3, Enter on a slice: check the frame `EDIT · BOUNDS` with start, end, length and cursor. Press K, then J/L.
- [ ] F12: check the frame `SETTINGS · AUDIO INPUT MIDI PATHS` and the `▸` row on every tab.
- [ ] F2, Enter on an empty pad: check its test tone with the envelope line in the strip.

### Listen

- [ ] `renders/98b-erase-pad-playback.wav`: low and high tones. In the second loop the eraser takes the low tone while it sounds, and the tone rings out. The third loop has only the high tone.
- [ ] `renders/98c-quantize-on-input.wav`: a low tone on beat 2 and a high tone on a 1/16 line between beats. Both loops sound the same.

## 98j: Controller map

Folder: `changes/archive/2026-11-12-98j-controller-map`

Connect the X-Session Pro. Run `task build`, then start `target/release/ptsr`.
These steps describe the implemented software paths. Leave every untested box open.

- [ ] Turn a knob. On F2, press F1, then Tab two times. Check the `CONTROLLER` tab and `[X-Session Pro]` on the top line.
- [ ] Check the drawing: deck A on the left, deck B on the right, four faders in the middle, the crossfader `<->` at the bottom.
- [ ] Check that each label agrees with the device: k1 `(volume)`, the left headphone button `[files]`, the right one `[fx]`.
- [ ] Move the selection with the arrows. Check that the bottom line names each control and its action.
- [ ] Press Esc, open F3 and open the tab again. Check the Chop labels: k1 `(start)`, k12 `(zoom)`.
- [ ] On F2, press Enter on a pad, then open the tab. Check the edit labels: k1 `(knob1)`, left LOAD `[back]`.
- [ ] Disconnect the device while the tab is open. Check `No MIDI device is connected.` and `X-Session Pro` under `Known profiles:`.
- [ ] Connect it again. Check that it comes back on the top line.

## 98k: Selected pad frame

Folder: `changes/archive/2026-11-12-98k-selected-pad-frame`

- [ ] Record A1 with hits on the selected pad and press Space. Check that the selected pad keeps its cyan frame on each hit, and only its inside flashes white.
- [ ] Turn marking on and mark the selected pad. Check the cyan frame around the yellow fill.

## 98l: A1 pattern overview

Folder: `changes/archive/2026-11-12-98l-pattern-overview`

The owner accepted the overview and authorized its PR and merge on 2026-11-12. The checks below remain unverified.
Use a disposable project or a copy. Start `target/release/ptsr` after `task build`.

### Keyboard and screen

- [ ] Record A1 hits on pads in banks 1 and 2. Press F5. Check the eight-bank index and 16 rows in the current bank.
- [ ] Check that `bank:count` means pads with A1 hits. Check long sample names and the bank-colored outlines at 80x26 and 120x40.
- [ ] Press U and O. Check that the selected bank section enters the viewport without changing the sound.
- [ ] Press Alt+H twice. Check that empty bank sections disappear and return while all eight bank numbers stay in the index.
- [ ] Press Alt+Up and Alt+Down. Check that the grid division changes without changing A1. Press Space and check the moving plate.
- [ ] Press F1 and read the mark legend. Press Esc to return to the overview. Press F2 to return to Pads.

### Controller and sound

- [ ] In F12 MIDI, learn `screen.pattern`, `pattern.overview_empty_toggle` and the two view actions on buttons.
- [ ] Use the learned buttons in the overview. Hit and release a Gate pad. Check its sound and release against Pads.
- [ ] Listen to the local `renders/97-pattern-record.wav`. The overview changes no sound and adds no WAV.

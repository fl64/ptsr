# Plan: cross-cutting work and recommendations (tools, safety, comfort, docs, risks)

Read `README.md` first. This document closes the gaps of the other plans. Some items are tasks
inside existing changes (the change number says where). Two items are new changes: `93a` and
`100a`. The letter keeps the numbers of the other plans valid.

## 1. Tools for the agent (the agent has no SMC-PAD and does not hear sound)

### 1.1 Virtual controller for live UX runs (task in change 91)

`AGENTS.md` lets the agent run the program in `tmux -L ptsr-ux` cut off from the owner's sound,
but the agent cannot press the owner's controller. Add a small dev tool:

- `task midi-replay -- <script>`: opens a virtual ALSA sequencer port named
  `SMC-PAD (virtual)` and sends the bytes of a script with delays. Script lines:
  `wait 200ms`, `cc 1 27 127` (channel, number, value), `note 10 48 100`, `noteoff 10 48`.
- The SMC-PAD profile matches the virtual port only when the run sets `PTSR_DEV_VIRTUAL=1`, so
  the owner's real device and the virtual one never mix.
- Scripts live in `tests/midi/*.txt`: one per user story of the controller paths (93). The agent
  runs them live and reads the screen with `tmux capture-pane -p`.

Check the license and the size of any crate you add for this (`midir` already has virtual ports
on ALSA; prefer it).

### 1.2 Screen catalogue (task in change 87)

`task screens` renders every screen in every mode to text files with ANSI colors:
`renders/screens/<screen>-<mode>.ansi`, at 120×40 and at 80×26. The owner looks at them with
`cat` in the terminal without starting the program. Each visual change refreshes the catalogue,
and its proposal names the files to look at. A test fails if a mode has no catalogue entry.

### 1.3 Owner check lists

Each change adds its hand and ear checks to `docs/LIVE-CHECKS.md` in the existing format, with
"keyboard only" and "SMC-PAD only" as two separate passes where the change touches input.

## 2. Safety on stage

### 2.1 Panic (change 93a)

One action stops all sound at once: transport stop, all voices fade out over the short cut
(`HushPads` exists; add the preview and the FX tails), recording stops and keeps what it has.

- Keyboard: `Ctrl+.` (right half; check clashes).
- SMC-PAD: STOP + PLAY pressed together (a chord: the second button within 100 ms, or PLAY
  while STOP is down). This does not open the panel.
- Notice: `! all sound stopped` (Warn).

### 2.2 A controller goes away while something is held (task in change 89 and 92)

- Unplug while STOP is held: the panel layer ends, the pads sound again. Unplug while a pad is
  held: the existing gate release happens (spec "75-unplug-gate").
- A layer or a hold never stays on without its button: the `Gestures` tracker clears all state of
  a port on disconnect and on a lost release (a press of the same control while it is "down").
- Notices: `· SMC-PAD connected`, `! SMC-PAD disconnected` (with what was released).

### 2.3 Two controllers at the same time (task in change 92)

The X-Session Pro and the SMC-PAD can be connected together (spec "Several MIDI devices at
once"). Rules:

- A layer (panel, erase, repeat) belongs to the device whose button made it. Pads of the other
  device keep sounding. The badge says which device: `PANEL · SMC`.
- The X-Session has no pads, so it has no panel. Its current physical map (change 82) must not
  change. Run its existing tests in every change of these plans; they are the regression guard.

### 2.4 Destructive actions

For the SMC-PAD, proposed default: clear pad, clear pattern, reset pad, apply a plan over
occupied pads and delete a slot open a question (`ASK`) in both panel forms. A latched
layer-2 cell needs no held STOP, so the panel alone is not a safety guard. Only an explicit
Yes cell confirms. Keep the X-Session direct safe defaults unchanged. Record the owner's
choice in change 92 before code. Existing keyboard behavior stays as specified.

## 3. Comfort (change 93a `93a-controller-comfort`, after 93)

### 3.1 Redo

There is undo (`Ctrl+Z`) but no redo. VISION asks for both. Add `edit.redo` for every step that
undo covers.

- Keyboard: `Ctrl+Y`, reserved by keymap cleanup 83a. After 83a, Chop hits use H and
  Shift+H. Check `config/keymap.toml`, owner overrides and the supported terminals before
  binding redo. The earlier Ctrl+U candidate is superseded.
- SMC-PAD: while STOP is held, LEFT is undo and RIGHT is redo (2 presses, symmetric, easy to
  find by touch). The Undo cell of the panel stays.
- A new edit after an undo drops the redo steps. Notice: `✓ undo: <what>`, `✓ redo: <what>`.

### 3.2 A new project without typing

The owner wants to start work from the controller. A new project needs a name, and names need
the keyboard. Add: the project picker has a row `+ new` that makes a project named by the date
(`2026-11-03-1`, then `-2`, ...). The owner renames it later from the keyboard, or never. The
same idea for pad names: they already come from the file name and the slice number.

### 3.3 Pad velocity curve (setting in F12 → MIDI)

SMC-PAD pads are small; a light hit can be too quiet. Settings per device: `linear`, `soft`
(more level for light hits), `hard`, `fixed 100`, `fixed 127`. Applied in the MIDI thread
before the hit goes to the engine. Render test: the same hits through each curve.

### 3.4 Controller settings in one place (F12 → MIDI)

Hold time (89), action echo on / off (92), encoder acceleration on / off and its strength (90),
velocity curve (3.3), the knob response (exists). Each with a knob path.

### 3.5 First connect and the cheat sheet

- The first time the SMC-PAD connects, one Info notice: `SMC-PAD: hold STOP for commands`.
  Kept in the config so it shows once.
- F1 gets a page `SMC-PAD` with the drawing of the device: buttons, the panel of the current
  screen, the two knob banks. Generated from the profile and `panel.toml`, not typed.
- `docs/SMC-PAD.md`: the same as a printable page for the owner, regenerated by
  `task controller-doc` (a test fails when it is out of date).

## 4. Additions to the sequencer plan

These are tasks inside the changes of `PATTERNS.md`:

- **95 transport:** tap tempo (`transport.tap`: the mean of the last 4 taps, a reset after 2 s;
  keyboard key in the design; SMC-PAD: a Pattern panel cell in 98, before it the bank 2 tempo
  knob is enough). The metronome level (a setting, a knob path). A project switch stops the
  transport and the recording first.
- **96 pattern play:** what a length change does: shorter — hits after the end are kept but not
  played (they come back when the length grows); show their count in the badge `2 hidden`.
- **97 pattern record:** a recording pass while the transport stops by itself (panic, project
  switch) is closed and kept as one undo step.
- **98 pattern screen:** step edits are undo steps (one per toggle; a quick series of toggles on
  one row within 2 s is one step). Redo from 93a works here too. The selected row can be a pad
  of another bank: `I` / `K` walk the rows of the pattern; LEFT / RIGHT on the bank row change
  the bank of the pad grid only.

### Change 100a — `100a-pad-mute` (after 100)

Pad mute while a pattern plays (VISION §12): a muted pad does not play its pattern hits; live
hits still sound. Keyboard: on the Pattern screen `Ctrl+M`? (Check the kitty protocol: Ctrl+M is
Enter in legacy terminals; pick another key if it clashes.) SMC-PAD: a Pattern panel latch cell
`Mute`: then pad hits toggle the mute of each pad. Visual: `MUTE` badge, a crossed pad glyph,
muted rows dim on the grid. Engine: a mute mask (`u128`) in the sequencer; a change acts at the
next block, no click (a voice already sounding is not cut).

## 5. Code structure (tasks inside the refactor)

`crates/app/src/main.rs` has about 20 600 lines. The refactor is the cheap moment to split it,
without behavior change:

- 85: move the input dispatch (`run`, `apply`, `midi_event`, `pad_hit`, `focus`) into
  `app/src/dispatch.rs`.
- 87: move the view model (what `App` gives to `tui`: status, badges, notices, hints) into
  `app/src/view.rs`.
- Later changes put new code in new modules (`pattern.rs`, `transport` state, `panel.rs`), not
  in `main.rs`.

Each move is its own commit with only the move in it.

## 6. Visual details that are easy to miss (add to change 87 and 92)

- **Kitty images and overlays.** The strip and the pad waves can be kitty graphics (F12 →
  Wave). A text drawn over an image (the value pop-up, the panel grid over the pad waves) may stay
  under the picture. Before an overlay is drawn on an area with an image, delete or hide the image
  there, and draw it again after. Test with kitty graphics on and off.
- **Small terminals.** The panel grid must fit where the pad grid fits (the 26-row rule of
  `screen-layout`). Cell labels have a short form of not more than 6 characters.
- **Blinks.** A queued slot and a count-in blink at 2 Hz, synced to the beat when the transport
  plays. Nothing else blinks.
- **Redraw.** Badges and the status line redraw only on a change, not every frame, to keep the
  UI thread light.

## 7. Documents to keep up to date in every change

`docs/USER-STORIES.md` (keyboard and SMC-PAD ways), `docs/CONTROLLER-PATHS.md` (from 93),
`docs/LIVE-CHECKS.md`, `docs/BACKLOG.md` (update rows 7 and 8 as their parts ship; do not
close sample BPM detection or pad loop mode when only transport and patterns ship),
`README.md` of the repository (the controller section), `config/keymap.toml` comments,
`docs/ui/STYLE.md` (87), `openspec/untested.md` for what needs the device.

## 8. Risks and what to do

| Risk | Sign | Plan |
|---|---|---|
| The SMC-PAD on Linux sends other bytes, or splits pads and knobs on different ports | step 0 of 91 | do step 0 now, in parallel with 84, without code; the owner needs 15 minutes |
| The encoders cannot be set to relative | step 0 of 91 | stop and ask; a weak fallback is "absolute as steps" (the difference of two values), which stops at the ends |
| The pad lights do not work on Linux | 94 step 0 | drop 94; the screen carries all state (VISUAL.md) |
| The refactor 85 changes behavior by accident | parity tests | small commits; the parity harness of 84 runs on each |
| Live hits feel late when recording | owner in 97 | change 101 |
| The UI thread is too slow with the new drawing | CPU of the UI, frame time | redraw on change only (§6); measure the frame time in the debug status |
| The audio callback gets slower with the sequencer | `task check` benchmark | add the sequencer with a full pattern (4096 hits) and 32 voices to the benchmark in 96; the limit stays 50 % of the buffer |

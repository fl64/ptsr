# Plan: screen structure, space, and bank mixer

This plan records the owner's review of the offline screen catalogue. It is a guide, not an active requirement.
Requirements come from `openspec/specs/` and the active OpenSpec change. This document changes no program behavior.
Keep the current dark palette, state accents, text badges, bank tints, fixed zones, and context strip.

Use one OpenSpec change at a time. Give each behavior change the next free number, its own branch, and its own PR.
Write its proposal, design, spec deltas, and tasks before code. Start the next change only after owner acceptance.
For each changed view, add catalogue fixtures at 120×40 and 80×26. Name the exact files in its proposal.
Use the sequence in section 5. The documentation split can run between two changes.

## 1. Current screens and checked limits

The owner reviewed `task screens` output at 120×40 and 80×26. The catalogue now has 458 ANSI files.
The implementation checks below qualify the visual findings where the code already has part of the requested behavior.

- F4 uses a small edit box at the upper left. The rest of the body has no content.
- F6 uses the shared pad frame. Its file list stays in the narrow right column.
  A selected file **already has a preview wave** in the strip, with its name and length.
  The list has no columns for rate, channels, or known tempo. The shared layout gives rows to the grid that the list needs.
- F2 uses the same frame. Its strip can repeat the selected pad's wave. Empty strip rows and low-terminal rows can take useful content.
- Empty pad cells show `empty` and can show a flat wave. This repeats across a fresh bank.
- Current labels include `PG n/a`, `scope pad 1`, a source ending in `· all`, the `A1` help group,
  an idle `clip` label, an idle `PLAY` slot, `No A1 · Ctrl+R record A1`, and `Alt N controls`.
  The transport help group is `CLOCK`. The MIDI learn label for `screen.pattern` is `rhythm`.
- Text clipping uses a grapheme boundary and `…`. It can cut a word.
- The pattern overview starts with `empty shown`. It draws all 16 pad rows in each visible bank.
  Hiding empty banks does not hide empty pad rows. One hit uses `■` at all velocities.
  Step widths vary because the layout distributes leftover columns among cells.
  Header numbers and hits use the same widths. Check their rendered alignment before changing that rule.
  The overview **already lights the playhead**. Preserve that behavior and test it during playback.
- Step edit can remove a selected hit. A pad hit on an empty cursor step does not add a hit.
- No screen shows the open bank's pad levels, sends, and mute states together.
  F4 does not show which pads feed each slot.
- `docs/ui/STYLE.md` contains the visual language and detailed rules for timers, queues, learning, and pictures.
  This makes the visual language hard to find.
- Factory keymap has no F8 screen binding. User keymap overlays can bind F8, so the mixer design must check conflicts.
- `docs/plans/V1.md` already plans pad mute as 100a. The mixer design must reconcile that work.

## 2. Rules for every change

- Keep the current colors, accent meanings, badges with text, bank tints, key buttons, and fixed status, title, notice, and help zones.
- Use all available rows at 120×40 and 80×26 when a larger element has content to show.
  Shorten text by priority, then end it with `…`. Do not cut a word.
- Show an element only while it adds information. Do not show a wave or meter as decoration.
- Use one name per concept on screens, in help, notices, MIDI learn names, and documents.
  The names are **transport**, **pattern**, **pad**, **slice**, **sample**, **bank**, **send**, **FX 1**, and **FX 2**.
  Show the slot name only with the concept, for example `pattern A1`.
  Add this glossary to `docs/ui/STYLE.md` and add a test for obsolete UI names.
- Every new view needs keyboard and controller paths, help groups, catalogue fixtures, and value pop-ups.
  Test keyboard and raw MIDI paths. Keep hardware-dependent results unverified until the owner checks the device.
- Keep one change per PR. Each proposal lists exact catalogue files to review at both sizes.
  Give separate keyboard and controller checks. Add tests for every spec scenario.
- Apply the file-growth rule from `AGENTS.md`. Put new screen code in modules.

## 3. Structure first

The catalogue at `477a169` is the base for this review. F5 and F7 still open modes inside
Pads. The owner accepted change 123, which addresses the overview rows, step widths,
velocity levels, and empty-step input. Do the structural changes after 123 and before
`screen-layouts`, `mixer-view`, and the 0.4b glitch kit. Give each change its own next free
OpenSpec number, branch, PR, specifications, scenario
tests, and catalogue fixtures at 120×40 and 80×26. Accept one change before the next.

Keep the current visual language, fixed zones, key-shaped pad grid, right pad inspector,
Chop screen, question layout, and filter curve. Files and FX still leave much of the body
empty. `screen-layouts` owns that work after the structural changes.

The archive of change 122 is complete. Its design assigns `No A1 · Ctrl+R record A1` to
`pattern-screen`, `PG on: needs linked slice` and `scope pad 1` to `pads-source-view`, and
`CLOCK` to `compact-help`. Check the affected catalogue entries when each change reaches
owner review.

### 3.1 Pattern track

The owner review of 2026-11-27 replaces the `pattern-screen` item with four changes.
[PATTERNS.md](PATTERNS.md#pattern-track-owner-review-2026-11-27) gives the checked
problems, the approved screen mockup, the two ways to write and the one undo.
[KEYMAP-CLEANUP.md](KEYMAP-CLEANUP.md#8-pattern-screen-keys-pattern-track) gives the keys.
[SMC-PAD-INTEGRATION.md](SMC-PAD-INTEGRATION.md#25-the-pattern-screen-pattern-smc) gives the controller model.

1. `pattern-screen-layout`: F5 opens its own Pattern screen. It has one header line,
   rows without bank frames and full-width cells. It has three velocity levels and the velocity lane.
   It has the detail line, the setup line and the empty state. View and live record only.
   Remove the old overview and the `Alt+N` panel. Pads keeps the mini lane.
2. `pattern-step-input`: step edit in the screen with pads = steps and pads = sounds,
   the pattern screen keys and one undo.
3. `pattern-smc`: the SMC-PAD model, its routes and device checks.
4. `help-tasks`: Help by tasks, as below.

Each proposal names the catalogue files at both sizes. It gives separate keyboard and
controller checks and names the renders to hear, or tells why the sound does not change.

#### Help by tasks (`help-tasks`)

The owner finding: F1 is one flat list. It starts with the Pitch Gate row. Undo and redo are
under CHOP, and tap tempo is under SOUND. It does not answer "how do I record a beat".
The proposal checks the row count and the groups in the catalogue.

- **Tab 1, THIS SCREEN, opens first.** It gives the tasks of the current screen and mode.
  Each task is a short recipe of one to three lines with the keys, in the order of use.
  Pattern tasks: "Record a beat", "Write steps", "Change one hit", "Erase a pad",
  "Length, tempo and grid" and "Undo". After an SMC-PAD input, the recipes name its controls.
- **Tab 2, KEYS.** First the keys of the current screen, then the global keys. Group them by
  task: play, record, edit, navigate and project. Use two columns at 120 columns. No paragraphs.
  Undo and redo are global. Tap tempo is in the Pattern group.
- **Tab 3, CONTROLLER** and **Tab 4, MESSAGES** stay as now.
- A long explanation, for example Pitch Gate, moves to a "more" line under its recipe.
  The line stays closed until the owner opens it.
- A test checks that each action of the current screen is in tab 1 or tab 2.
  It also checks that no action is in the group of another screen.

`compact-help` (§3.3) changes the help band at the bottom. `help-tasks` changes F1 only.

### 3.2 `pads-source-view` (change 139)

The top strip of F2 Pads has a Sound view and a Rhythm view. Sound shows the full
sample wave, the ranges of the pads, and a playback line for each sounding voice.
The last wave stays visible in silence. A pin keeps focus on the selected sample.

Show one other sounding sample at 80x26 and up to two at 120x40. Give each a
compact row. Show a count and the pad labels for samples beyond those rows.
Keep all new playback indicators in the strip. Rhythm uses the A1 event rows.
Recording and pad edit take priority when active.

Alt+9 changes the strip. Alt+0 pins the selected sample. In the SMC-PAD panel,
P16 changes the strip on Pads and pins the sample on Pad Edit. Change 139 defines
the input paths, focus timing, and screen checks.

### 3.3 `compact-help`

Show one help row with the most-used keys for the current screen and mode. Show the full
three-row band while a modifier is held where release events exist, and in F1. Add a
setting that keeps the full band visible. Default to one row. Test both terminal sizes,
mode changes, modifier release, and the setting. Name compact and full-band catalogue
files and separate keyboard and controller checks in the proposal.

Settings uses the active tab for its short hint and F1 keys (157). The folder picker names browse, choose, and cancel.

### 3.4 `key-consistency`

Use U/O for banks on every screen with a pad grid. Keep PgUp/PgDn as a second bank path
on all those screens. Remove Files folder-up from J. Choose Backspace or Esc and define
the list-root behavior in the design. Keep J/L for less and more when a value is selected.
Show moved-key notices for one release. Test keyboard and raw MIDI paths, and name the
affected catalogue files at both sizes in the proposal.

One F key opens one screen. Modes stay inside a screen and show their badge. A new view
reserves space in the layout spec before a later feature uses it.

## 4. Ordered changes

### 4.1 `ui-quiet-and-words`

Change 122 is complete. Section 3 assigns its deferred labels to later changes.

Make empty pads quiet: show a muted `—`, with no wave line and no `empty` word.
Keep a pad-mode corner mark, including `◎`, on an empty pad when it has meaning.

Replace unclear labels across the screen, help, notices, and MIDI learn names:

- Show `pitch gate on/off` only when pad pitch differs from zero.
- Show scope as `edit: this pad`, `edit: marked (3)`, or `edit: bank`.
- Show a sample source as `kick.wav · whole file` or `kick.wav · slice 3`.
- Name the `A1` help group `PATTERN`.
- Show `clip` in magenta only after clipping, with the time of that event. Hide the idle label.
- Leave the idle title play slot empty. Preserve the active play state and its fixed zone.
- Show `pattern A1 is empty · Ctrl+R records` when the pattern has no hits.
- Replace `Alt N controls` with `Alt+N: pattern keys`, if the action still opens pattern controls.

Apply the section 2 glossary. Give long labels a priority order and a word-safe `…` ending.
Check both catalogue sizes for cut words, including Chop and 80-column help.
At 80×26, give otherwise empty rows to the grid or right column.
Add tests for the labels, empty-pad mode marks, clipping state, and used rows.

### 4.2 `pattern-view-polish`

The owner accepted change 123. Its archive is part of [PR #8](https://github.com/fl64/ptsr/pull/8).

Show pads with hits plus the selected pad by default, in bank order, with bank labels.
`Alt+H` shows every row. Keep this toggle visible in help and the catalogue.
Use a fixed width for each step cell. Put each header number over its cell.
Keep beat and bar shading. Use a separate even grid for triplet divisions.
Test cell and header positions at both catalogue sizes and at each supported division.

Show soft, medium, and loud hits with three brightness levels of one glyph.
Keep multiple-hit and off-grid information visible. Set velocity thresholds in the design.
Keep the existing lit playhead while the transport plays, and add a playback fixture.

In step edit, a pad hit adds its hit velocity at the cursor step.
Shift gives full keyboard velocity. A hit on an occupied step of that pad removes its hit.
`J` and `L` move the cursor. Undo restores either edit.
The design must define which hit the editor removes when a cell has multiple hits.
Update the `patterns` spec and test raw MIDI velocity, keyboard Shift, add, remove, and undo.
End long pad names with `…` at a word boundary.

### 4.3 `screen-layouts`

Let each screen allocate its body inside the fixed zones.

- **Files (F6):** give most of the width to a file list with name, length, rate, channels,
  and tempo when known. Keep the existing selected-file preview wave in the strip and make its source clear.
  Hide the pad grid when no file is in hand. Show the grid while placing a file.
  Preserve the keyboard and controller path to pads in hand.
  A successful pickup replaces the item in hand. A refused pickup keeps it.
- **FX (F4):** show both slots large and side by side, with all current parameters.
  Add a return meter for each slot. Show a send matrix for the open bank:
  up to 16 pads with sound by FX 1 and FX 2, with short bars and values.
  Select a matrix cell and use `J` or `L` to change that pad's send.
  State the keyboard and controller paths to each cell in the design.
- **Pads (F2):** keep the context lane from section 3.2. Give it the rows it needs inside
  the shared screen layout.
- **Meters:** show L and R in one row each, with dB values.

Test layouts and usable space at both catalogue sizes.
Include an empty bank, a loaded pad, a file in hand, long file names, and each pad edit page.

### 4.4 `mixer-view`

Add a bank mixer screen. Show all 16 channels of the open bank.
At 80 columns, show eight channels per page and use Left/Right or `U`/`O` for the other eight.
Each channel shows its name, volume, FX 1 send, FX 2 send, mute, and solo.
Add pan after `pad-sound`. Add a meter after `pad-bus`.
Omit those fields if the mixer comes before those changes.

Use F8 if the design finds no factory key or user overlay conflict.
`I`/`K` select the row or parameter. `J`/`L` change its value.
Left-hand pad keys select a channel and sound its pad, as on Pads.
For SMC-PAD, the eight knobs of bank 1 control the current parameter of the eight visible channels.
Left/Right page the channels. A panel cell opens the mixer.
Keep the actual SMC-PAD profile unverified until the owner measures the hardware mapping.
Show a help group and value pop-up for every changed value.

Add solo. A soloed pad silences other pads' dry sound and new sends.
Existing FX tails continue to ring. Save mute and solo in the project and migrate older projects.
The design must resolve the planned 100a pad mute before implementation and must define solo with multiple soloed pads.
Test dry sound, sends, tails, project migration, and keyboard and MIDI control paths.

### 4.5 `style-doc-split` (between two changes)

Keep `docs/ui/STYLE.md` to one page: colors and bank tints, accent meanings, badges,
zones, the glossary, key button style, and the rule that color never stands alone.
Move timer, queue, learning, picture, and catalogue rules to the matching
`screen-layout` or `screen-help` specs, or to `docs/ui/RULES.md` when they are guidance.
Link the new locations from `STYLE.md`.
The PR must map each old rule to its new place. Do not lose a rule or change behavior.

## 5. Order with the FX and filters plan

Use this proposed sequence across the two plans. Complete and accept one change before the next.
The order within [FX-FILTERS.md](FX-FILTERS.md) and sections 3 and 4 above stays fixed.

1. `split-modules` (121) and the 122 UI words change are complete.
2. The owner merged `repo-fast-checks` and accepted `pattern-view-polish` (123).
3. Do `denormals` next. Record this order
   in its proposal. Keep the 94a callback peak investigation open until its criteria pass.
4. Do the pattern track of §3.1 (`pattern-screen-layout`, `pattern-step-input`,
   `pattern-smc`, `help-tasks`), `pads-source-view` (including `pads-context-lane`),
   `compact-help`, and `key-consistency` in that order.
5. Do `style-doc-split` after the glossary exists and before `screen-layouts`.
6. Do `screen-layouts` next. Keep its FX geometry ready for later effect parameters.
7. Resume the FX order: `metronome-path`, `fx-cost`, `master-safety`,
   `pad-sound`, `pad-reverse`, and `pad-bus`.
8. Do `mixer-view` after `pad-sound` and `pad-bus`, so its first version has pan and meters.
   Resolve 100a pad mute in its design before code.
9. Complete the remaining FX changes: `drive-aa`, `filter-v2`, `reverb-v2`, and `delay-v2`.

Do not run two active behavior changes at once.

## 6. Owner decisions before implementation

| Before | Question | Proposed default |
| --- | --- | --- |
| 3.1 | Which key switches view and edit inside Pattern? | Enter. |
| 3.1 | Does the screen say `pattern`, with no `A1` until slots exist? | Yes. Owner default, 2026-11-27. |
| 3.1 | What is the default in step edit? | Pads = steps. Owner default, 2026-11-27. |
| 3.1 | One undo key for pattern and other edits? | Yes. Owner default, 2026-11-27. |
| 3.1 | Remove the old Alt chords after one release? | Yes. Owner default, 2026-11-27. |
| 3.1 | Does SMC-PAD knob 2 select the row in step edit? | Yes. Owner default, 2026-11-27. |
| 3.1 | Does Help open on the tasks of the current screen? | Yes. Owner default, 2026-11-27. |
| 3.1 | Metronome on `M` breaks the mark-key rule. Which key? | Keep `Alt+M`. Owner decision, 2026-11-27. |
| 3.2 | Does Pads show a mini pattern lane when a pattern exists? | Yes. |
| 3.2 | Is source the default strip view on F2 while playing? | Yes. |
| 3.2 | Does source focus follow the last hit or the selected pad? | Last hit, with a pin for the selected pad. |
| 3.2 | What is the default source refresh rate? | 30 frames per second. |
| 3.2 | How many other sample rows can the strip show? | Two at 120×40 and one at 80×26. |
| 3.3 | Which help band is the default? | Compact, one row. |
| 4.2 | How do hit velocities look? | Three brightness levels of one glyph. |
| 4.3 | Does Files show the pad grid with no file in hand? | No. The file list uses that space. |
| 4.4 | Which key opens the mixer? | F8, after keymap conflict checks. |
| 4.4 | Does solo silence old FX tails? | No. It silences other pads' dry sound and new sends. |
| 4.4 | How does mixer mute relate to planned 100a? | Use one pad-mute behavior. Resolve ownership in the mixer design. |
| Cross-plan | When does FX work resume? | Do `denormals` after 123 and `repo-fast-checks`, before `pattern-screen`. Resume other FX work after `screen-layouts`. |

These defaults are proposals, not owner acceptance of the future OpenSpec changes.

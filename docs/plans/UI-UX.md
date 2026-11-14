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
Pads. Change 123 addresses the overview rows, step widths, velocity levels, and empty-step
input. Before acceptance, check 123 against the required `Specs` job, the scenario tests,
and the current PR body template. The owner has not accepted 123 yet. Do the structural
changes after that acceptance and before `screen-layouts`, `mixer-view`, and the 0.4b glitch
kit. Give each change its own next free OpenSpec number, branch, PR, specifications, scenario
tests, and catalogue fixtures at 120×40 and 80×26. Accept one change before the next.

Keep the current visual language, fixed zones, key-shaped pad grid, right pad inspector,
Chop screen, question layout, and filter curve. Files and FX still leave much of the body
empty. `screen-layouts` owns that work after the structural changes.

The archive of change 122 is complete. Its design assigns `No A1 · Ctrl+R record A1` to
`pattern-screen`, `PG on: needs linked slice` and `scope pad 1` to `pads-context-lane`, and
`CLOCK` to `compact-help`. Check the affected catalogue entries when each change reaches
owner review.

### 3.1 `pattern-screen`

Make F5 open a separate Pattern screen. Keep overview and step edit as modes inside that
screen. F7 becomes free. Use Enter to switch between view and edit by default. Record the
key choice and the moved-key notice in the design and keymap. Show `F5 PATTERN` in the title.
Keep pattern data and saved files unchanged.

The proposed layout keeps one row for the selected step value:

```text
 F5 PATTERN  A1 · 2 bars · 1/16 · ▶ 1.3       bar 1 of 2
          1 . . . 2 . . . 3 . . . 4 . . .
 1 kick   █ · · · · · █ · · · · · █ · · ·
 2 snare  · · · · █ · · · · · · · █ · · ▪
 3 hat    ▪ · ▪ · ▪ · ▪ · ▪ · ▪ · ▪ · ▪ ▪
 Q chop3  · · · · · · ▪ · · · · · · · · ·
          ▲
 vel      ▆ ▁ ▃ ▁ ▇ ▁ ▆ ▁ ▃ ▁ ▃ ▁ ▇ ▁ ▃ ▅
```

- Show pads with hits plus the selected pad in bank order, with bank labels. Add a row when
  its pad gets a hit. Alt+H shows all rows and defaults to off.
- Give each step an equal-width cell. Put the header number over its cell. Keep beat and bar
  shading. Use an even grid for triplet divisions. Test column positions at both sizes.
- Show soft, medium, and loud hits with three brightness levels of one glyph. Light the
  current step column during playback.
- Reserve one row below the grid for the selected row's step value. Show velocity now.
  Later 0.4b changes can use this row for locks, ratchets, and probability.
- In edit mode, a keyboard or MIDI pad hit adds a hit at an empty cursor step with its input
  velocity. Shift gives a keyboard hit full velocity. A hit for the same pad at an occupied
  step removes that hit. Undo restores each edit. J/L move the cursor. I/K select a row.
- Give the controller a panel action for the Pattern screen, LEFT/RIGHT bar pages, and knobs
  for the step value. Record that the physical SMC-PAD mapping still needs measurement.

The new proposal must name view, edit, triplet, and moving-playhead catalogue files at both
sizes. It must give separate keyboard and controller checks and name the renders to hear.

### 3.2 `pads-context-lane`

Use the top strip of Pads for information that the grid and inspector do not show.

- When a pattern exists, show a small lane for its used pads and the moving playhead. Show
  only as many rows as fit.
- In pad edit, show the selected region with attack, decay, and release on its wave. Keep
  the filter curve on the filter page. Show sends on the FX page.
- When the strip adds no information, reduce its height and give the rows to the grid. Do
  not leave empty strip rows.

The proposal must name pattern, region, filter, sends, and empty-strip catalogue files at
both sizes. It must give separate keyboard and controller checks and name the renders to hear.

### 3.3 `compact-help`

Show one help row with the most-used keys for the current screen and mode. Show the full
three-row band while a modifier is held where release events exist, and in F1. Add a
setting that keeps the full band visible. Default to one row. Test both terminal sizes,
mode changes, modifier release, and the setting. Name compact and full-band catalogue
files and separate keyboard and controller checks in the proposal.

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

Change 123 is in owner review. Apply the current `Specs` and scenario checks before acceptance.

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
2. Review 123 with the current spec and PR checks. Accept `pattern-view-polish` after the
   owner checks its screen catalogue, input paths, and render.
3. Do `pattern-screen`, `pads-context-lane`,
   `compact-help`, and `key-consistency` in that order.
4. Do `style-doc-split` after the glossary exists and before `screen-layouts`.
5. Do `screen-layouts` next. Keep its FX geometry ready for later effect parameters.
6. Resume the FX order: `denormals`, `metronome-path`, `fx-cost`, `master-safety`,
   `pad-sound`, `pad-reverse`, and `pad-bus`.
7. Do `mixer-view` after `pad-sound` and `pad-bus`, so its first version has pan and meters.
   Resolve 100a pad mute in its design before code.
8. Complete the remaining FX changes: `drive-aa`, `filter-v2`, `reverb-v2`, and `delay-v2`.

If callback peaks block screen work, move `denormals` ahead of step 2.
Record that decision in the first active proposal. Do not run two active changes at once.

## 6. Owner decisions before implementation

| Before | Question | Proposed default |
| --- | --- | --- |
| 3.1 | Which key switches view and edit inside Pattern? | Enter. |
| 3.1 | Does Pattern show empty rows by default? | No. |
| 3.2 | Does Pads show a mini pattern lane when a pattern exists? | Yes. |
| 3.3 | Which help band is the default? | Compact, one row. |
| 4.2 | How do hit velocities look? | Three brightness levels of one glyph. |
| 4.3 | Does Files show the pad grid with no file in hand? | No. The file list uses that space. |
| 4.4 | Which key opens the mixer? | F8, after keymap conflict checks. |
| 4.4 | Does solo silence old FX tails? | No. It silences other pads' dry sound and new sends. |
| 4.4 | How does mixer mute relate to planned 100a? | Use one pad-mute behavior. Resolve ownership in the mixer design. |
| Cross-plan | When does FX work resume? | After the small UI steps and `screen-layouts`. Move `denormals` earlier if peaks block work. |

These defaults are proposals, not owner acceptance of the future OpenSpec changes.

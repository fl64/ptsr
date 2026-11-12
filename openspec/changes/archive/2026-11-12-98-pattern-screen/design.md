# Design

## Context and owner decisions

The owner tried the current recording flow. Recording looks acceptable to the owner, but held erasure is unclear.
The independent review used existing screens and source, not a new execution or a human test.
Its observation about missing event positions agrees with the owner's reported difficulty.

The owner accepted an event strip under the large wave and a time-position-first editor.
Physical pads keep their sound identities. They do not become numbered time steps.
The owner approved continuation of 98 on the current `97-pattern-record` branch before 97 closure.
Preserve all existing work. Keep 96, 97 and 98 evidence separate.
No permission exists for commits, publication, archive, merge, live audio/MIDI or owner-project changes.

The agreed first scope is removal of existing events, not insertion or a complete piano-roll editor.
The owner asked for different step divisions. The 16-cell drawing is an example, not a storage limit.
Existing 4/4 transport remains. No answer approved another meter.

## Scope after the owner check

On 2026-11-11 the owner checked the rhythm view and asked for a convenient and complete result.
Change 98 now closes with the view only. The step editor moved to the deferred draft [98a step edit](../../../docs/plans/98a-step-edit.md).
The editor sections below stay as background for that draft. They are not requirements of change 98.

## Approved first implementation increment

The owner approved the bounded first increment: working F2 event view, tests and F7/F8 view controls.
It does not include event removal, edit entry, cursor commands or a placeholder F5 binding.
Keep view state outside saved data and use the existing compiler and audio feedback.
Change only the approved App view/input/catalogue files, shared input and terminal view, shipped keymap and these progress artifacts.
Do not change audio processing, project format or the unfinished 96/97 implementation.
The full active-scenario audit remains incomplete until later editor scenarios have real implementation tests.
Do not mark the change complete or add scenario exceptions for this intermediate view.

## Verified implementation boundaries

- `crates/engine/src/pattern.rs`: view divisions have exact tick widths. Compilation can retain source-row descriptors through sorting and quantize collisions.
- `crates/engine/src/capture.rs`: phase-preserving publication accepts an unstamped replacement while capture is Idle.
- `crates/engine/src/sequencer.rs`: that publication keeps existing note deadlines instead of releasing every sequenced Gate.
- `crates/input/src/lib.rs`: shared focus and pad routing need an explicit silent editor role. A UI-only mode flag would not prevent direct MIDI sound.
- `crates/tui/src/lib.rs`: Pads uses terminal-cell waves. The draw dispatcher sends its kitty image collection only to Chop.

These are source observations, not proof of editor implementation or physical-device behavior.

## User contract

A recorded A1 plays. The owner sees a pad's rhythm, selects a time cell, and removes one existing event through its pad.
Other raw rows, their timing, velocity, Gate lengths and pattern settings remain unchanged.
Undo restores that removal. The playing clock does not follow the edit cursor.

Separate three quantities:

1. Raw timing: the stored take, unchanged by viewing.
2. Playback timing: raw timing after the existing quantize and strength rules.
3. View division: the cells used to inspect playback timing, independent of quantize.

## Event strip and layout

Use the existing full-width F2 strip, whose normal wave shows live output.
Reserve three rows at its bottom for pattern information: the header, the all-pads row and the selected pad's row.
Keep the shared outer frame and pad rows fixed.
With four or more strip rows, retain at least one wave row above the event rows.
At 80x26 the strip has three rows. There the event rows replace the wave. The L/R level meters in the right column still show output level.
Other screens retain their existing strip contents and dimensions.
Existing pad-parameter curves, splash, count-in and overlays retain priority in their own regions.
F2 currently draws its wave as terminal cells. Only Chop collects kitty images in the shared draw dispatcher.
Keep that boundary. Do not add a new F2 image renderer for this editor.
Old images from a screen change must not cover event text.

Write the header in words: `A1 · bar 1/2 beat 3 · 1/16 · pad 1 kick`.
The owner rejected the earlier abbreviated header `A1 b1/2 1/16 s1-16 B1/1 kick` as unreadable.
Under the header, show the all-pads row and the selected pad's row. Each row starts with a short label: `all` and the pad name.
The all-pads row marks a cell when any pad of the current bank has an onset there. It shows the whole beat of the bank at once.
It does not show other banks. A separate request to show activity of other banks during Play stays open.
Use distinct empty, on-grid, off-grid and multiple-event forms. Draw empty cells dim and onsets bright.
Group cells by quarter-note beats. Do not assume four cells per beat.
Each beat starts with one space. Each cell then takes its glyph and one space, so every glyph has a space on both sides.
Alternate beats use a darker background. A vertical line separates bars. Do not draw lines between beats.
At 1/16 a bar takes 36 columns, so a two-bar loop with labels fits 80 columns.
Show the playback position without advancing a UI clock. Use a coherent current-project audio snapshot.
Cursor trials: the owner rejected the white/yellow V1 (low contrast) and found the white inverse plate readable but not attractive.
The inverse plate covered a leading space and its glyph, so the glyph sat off center.
The owner then delegated the choice to the agent after offline mock-ups. The selected style is a centered plate.
Draw the current cell in both event rows as three columns: the glyph and the spaces on both sides.
Use a bold dark glyph on a light gray background. Apply it to empty, aligned, off-grid and multiple-event cells.
Do not draw an arrow, frame or additional cursor row for playback. Do not add red.
The mock-ups also showed flicker. The cause was the mock-up script, which cleared each line before redraw.
The program draws through ratatui, which writes only changed cells. Its behavior in the owner's terminal needs an owner check.
The separate edit-cursor outline remains required below. The read-only increment does not implement that outline.
When the audio snapshot is unavailable or stale, do not invent a moving marker.
A missing or out-of-page position leaves all cells in their ordinary style. A fresh frame must remove the previous inverse position.
With no A1, show `No A1` and the effective recording action. Do not create a pattern by viewing it.
An existing empty A1 shows empty cells without a dirty mark or Undo entry.

Outside edit mode, a running pattern selects the page containing the playhead.
While stopped, show the first page unless the runtime view already has an explicit edit position.
In edit mode, the edit cursor controls the visible page. Playback never pulls that page away.
If playback is outside the visible page, show its bar/step as text without placing a false marker in the page.

The two position markers must differ by glyph or outline, not color alone.
Show the `STEP EDIT` badge and yellow pad-mode frame only while editing.
Ordinary playback flashes remain playback feedback, not evidence of a stored event at the edit cursor.
In edit mode, onset presence has a separate pad marker. A held Gate is not a new onset in every following cell.
With a hidden pad-grid preference, edit mode temporarily shows the grid. Exit restores the preference without saving a visibility change.
The event detail area shows selected event index/count, raw and playback positions, velocity and Gate length without changing them.

### Bank row

The owner reported that hits on other banks are not visible during Play. The owner approved this on 2026-11-11.
Underline the number of each bank with a visible A1 onset.
Light the pads of closed banks in their two-glyph pad map during the current 1/16 playback step of their onset. A fixed 1/16 step keeps a flash visible at fine views.
The flashes use the same audio position as the event strip. The open bank keeps its existing flashes, and the bank never changes by itself.

## View divisions and pages

The view supports 1/4, 1/8, 1/8T, 1/16, 1/16T, 1/32, 1/32T and 1/64. It defaults to 1/16.
Off is a playback-quantize choice, not a cell width. Off retains the last view division.
Changing the view never changes quantize, strength, raw hits or the dirty state.
The view state is runtime-only and resets to 1/16 for a new project or restart.

A page shows as much of the loop as fits the width. It holds the largest count of whole bars that fits, up to the whole loop.
Pages start at bar lines. The last page can hold fewer bars. If one bar does not fit, a page holds half a bar, then one beat.
Columns per bar are 4 × (1 + 2 × cells per beat). Each line between two bars takes one more column. The row labels take five columns.

| View | Cells per 4/4 bar | Columns per bar | Bars per page at 80 / 120 columns |
| --- | ---: | ---: | --- |
| 1/4 | 4 | 12 | whole loop up to 5 / 8 bars |
| 1/8 | 8 | 20 | 3 / 5 bars |
| 1/8T | 12 | 28 | 2 / 4 bars |
| 1/16 | 16 | 36 | 2 / 3 bars |
| 1/16T | 24 | 52 | 1 / 2 bars |
| 1/32 | 32 | 68 | 1 / 1 bar |
| 1/32T | 48 | 100 | half bar / 1 bar |
| 1/64 | 64 | 132 | half bar / half bar |

These counts are calculations for the full screen width. The implementation uses the actual strip width.
Pages contain complete beats. Do not split six-cell or twelve-cell triplet beats.
A cell covers the half-open interval from its grid line to the next grid line.
Use the compiler's playback tick to place each visible raw row in a cell. Do not round it to the view grid.
An onset inside the cell but not on its first tick has an off-grid marker.
A cell with multiple source rows shows multiplicity, including rows that quantize onto the same tick.
Do not deduplicate the stored take to draw a cell.
Change 97 replaces same-tick rows of one pad when a pass completes. A multiple-row cell then usually shows rows at different ticks of a coarse cell, or rows from a loaded project.
Filter hidden raw hits before applying playback quantize, as change 96 requires.
A visible hit quantized to the loop seam appears at the start, not on a nonexistent end cell.

Cursor movement spans the whole loop and clamps at its first and last cells. It does not wrap into another bar unexpectedly.
On a view change, keep the old cell's starting tick and select the new cell containing that tick.
Clear any event selection whose pad, identity or cell membership no longer matches.
Bank changes keep the time position but select the corresponding pad in the new bank.

## Edit entry, input and lifecycle

F5 on Pads toggles `pattern.step_edit`. It is a mode switch, not `screen.pattern`.
Entry requires an existing A1, closed pad edit and no incompatible input owner.
Refuse entry during CountIn, Recording, Finishing, pending capture or Undo, project restoration, placement, marking, pattern controls or audio recording.
Refusal names the blocking work and the next action. It does not end that work or create A1.
A playing A1 may enter edit mode without stopping or resetting transport.
First entry selects the current playing cell from valid audio feedback, or the first cell if no such feedback exists.
Later entry in the same project retains the last edit position.

J/L or Left/Right move the selected step. I/K or Up/Down select an event in the selected pad's cell.
Tab and Alt+Up choose the next (finer) view division while editing. Shift+Tab and Alt+Down choose the previous division.
Alt+Up/Alt+Down also select the view on ordinary Pads, so the read-only increment has a keyboard path.
The first increment used F7/F8. A screen binding outranks an owner's global binding of the same key, so F7/F8 hid an owner's global F8.
On 2026-11-11 the owner approved a move to free keys. The agent rejected Alt+Left/Alt+Right: on Chop they move the slice end, and a chord keeps one role.
Alt+Up/Alt+Down are free on every screen. Their role matches Up/Down zoom on Chop: a finer view zooms in.
View choices follow the table order and clamp at its ends. The table is in order of cell size.
The first order put the triplet views after 1/64, so Alt+Up after 1/64 went to a larger cell. The owner saw this on 2026-11-11 and approved the cell-size order.
The header writes the view in words: `grid 1/16`, `grid 1/16 triplet`.
F5 or Esc closes the editor. Opening another screen closes it without undoing accepted removals.
F1 suspends edit input and returns to the same editor position when closed.
Pads remain silent under F1 while this mode owns them. Closing help does not replay those presses.
Text, pickers, questions and learn retain their existing source-specific priority.
Selecting a command for learning must not execute it.

Pad presses in edit mode retain their ordinary absolute-pad addresses but route to App without immediate sound or capture.
No plain or Shift pad-block key becomes a navigation command. Shift does not change removal behavior.
An edit press removes at most one event. Keyboard repeats and pad releases never remove another event.
Leaving edit mode while the owner holds an edit pad must not start a delayed sound.
A Gate pressed before entry still releases its original voice, even after a bank, mode or screen change.
Apply the same rules through keyboard and raw MIDI production routing, not a test-only equivalent.

Refuse incompatible recording, placement, marking, pattern-control and pad-parameter requests while editing, with an exit instruction.
Do not silently leave edit mode and interpret the same pad press as sound or assignment.
Transport start/stop, tempo, metronome, bank changes and ordinary save remain available.
An accepted project transition closes and resets the editor. A refused transition keeps it and its work.
Old-project deferred commands and old row identities cannot edit the new project.

## Event choice and removal

Compute cell membership from the current checked pattern and compiler descriptors outside the audio thread.
Keep raw row identity through sorting, quantize collisions and removal. An array index alone is not a durable event identity.
A loaded pattern may receive runtime row identities, but that must not change saved hits or create a user edit.

For a cell with one event for the pressed pad, remove that event directly and select that pad.
For an empty cell, select the pad without adding an event, changing history or clearing its sample.
For a different pad with multiple events, the first press selects that pad and its first event without removing anything.
For the selected pad, I/K chooses among its events. A following pad press removes only the selected event.
Order choices by playback tick, raw tick and original row order. Show the selected event and count before repeated removal.
After removal, retain another valid event selection or show an empty cell. Never select an unrelated cell's event automatically.
Rows at the same compiled tick remain separate choices. Removing one may leave another audible event there.
Explain that remaining event through its count and lit pad state, not a false empty cell.

Revalidate the chosen identity, current project, pad and cell before mutation.
If another operation invalidated it, refresh the selection without removing a replacement row.
Build and check the replacement before committing the App edit and its Undo entry.
Use existing phase-preserving pattern publication, not ordinary replacement that releases all active sequenced Gates.
A removal affects future onsets after the accepted audio boundary. Already-started notes retain their existing deadlines and tails.
Queue and retirement pressure may defer publication, but cannot lose a removal, duplicate it or acknowledge a failed save as successful.
Do not mark sound as changed before the replacement can reach the current project's engine.

## Undo, save and sound boundaries

Each successful removal is one common-history Undo step, with raw pattern and row descriptors.
Ctrl+Z and existing learned `edit.undo` undo that removal when it is the newest common-history edit.
Existing `pattern.undo` may undo the newest pattern edit without crossing a newer pad or FX edit.
Its help must describe both recording-pass and single-event scope rather than always claiming a whole pass.
Empty presses, focus, cursor and view changes do not create history or dirty state.
Keep current history capacity, save refusal, autosave and project-transition policies.
Save the edited raw rows through format 5. Do not save editor state or change unaffected fields.
Restart leaves transport stopped and editor closed.

No sound algorithm, new DSP, callback storage, dependency or system policy change belongs to this change.
Use existing publication and retirement rules. Preserve no-allocation tests and the original callback benchmark gates.
A new checked render demonstrates original rhythm, one removal and Undo with another pad unaffected.
Historical maximum failures remain unresolved even if another check passes.

## Edge cases

The statuses below are design dispositions, not passed implementation tests.

| Situation | Required result | Status | Planned check |
| --- | --- | --- | --- |
| Off or partial quantize, different view | Preserve raw timing and show between-line onsets | Closed in contract | All views at strengths 0, 50 and 100 |
| Triplet view and page boundary | Keep complete beats and exact cell membership | Closed in contract | Every page at 1/16T and 1/32T |
| Same-pad events share a cell or tick | Select and remove only one identified raw row | Closed in contract | Multiple off-grid rows and quantize collisions |
| Short loop hides raw rows, seam quantize wraps one | Hidden rows stay hidden. Visible seam hit appears once | Closed in contract | Shorten/grow and last-tick fixtures |
| Held Gate crosses entry, bank or exit | Original release works. Edit press never sounds later | Closed in contract | Keyboard and raw MIDI sequences |
| Recording command precedes an edit press | Refuse incompatible mode. Retain input causal order | Closed in contract | Pending and accepted capture phases |
| Project or pattern changes before queued removal | No stale identity can remove a replacement row | Closed in contract | Epoch/revision change before dispatch |
| Publication or retirement pressure | Retain one ordered edit and its Undo without callback blocking | Closed in contract | Bounded-queue and garbage-pressure fixture |
| Existing Gate rings during removal | Keep its deadline and unrelated tails | Closed in contract | Offline render and allocation checks |
| Kitty image and 80x26 safety overlays | Event rows stay legible. Save/audio warnings retain priority | Closed in contract | Offline geometry and owner graphics check |

Physical-controller ergonomics and actual kitty composition remain open until owner checks.

## Alternatives not selected

A help-only change is smaller but leaves the owner timing erasure against sound without an event target.
A read-only strip improves orientation but cannot by itself remove one wrong hit without timing a hold.
The accepted design therefore separates the strip from the next removal increment.

The older selected-sound/time-step-pad model changes every pad's role and does not match the owner's selected-position proposal.
A full F5 table, insertion, Accent/Ghost presets and a new meter would increase scope before the owner checks the correction flow.
Keep them outside this first editor.

## Open questions

The owner approved the visual model and combined working branch, not every control and boundary default below.

- Are F5 edit toggle, Alt+Up/Alt+Down view, J/L step and I/K event selection comfortable? Default: use these free or contextual command roles.
- Is the first-focus rule for another pad's ambiguous cell clear? Default: focus first, then remove one chosen raw event.
- Should empty cells insert events? Default: no insertion in this scope.
- Do different beats include 3/4, 6/8 or 7/8? Default: 4/4 only until explicit scope approval.
- Should view and cursor survive restart? Default: runtime-only, with 1/16 and closed edit on load.
- Where should the first edit cursor start? Default: current playback cell when available, otherwise the first cell. Reentry retains the cursor.
- Should removing a started Gate cut it? Default: affect future onsets only and preserve current note ownership and deadlines.
- Is immediate live editing preferable to stop-only correction? Default: allow Idle recording with either playing or stopped transport.
- Should the bank row light manual hits on closed banks? Default: no. It lights A1 onsets only, from the pattern and the audio position. Manual hits on closed banks would need per-voice activity from the audio thread.
- At 80x26 the strip has three rows. Should the event rows replace the wave there? Default: yes, the L/R meters still show level.
- Should the all-pads row show other banks during Play? Default: no, current bank only. The separate bank-activity request stays open.
- Does the playback plate move too often at 8 steps per second? Default: move it per cell. Alternatives: move per beat, or tint the played part of the bar.

If implementation contradicts a requirement, stop and propose a requirement edit before code continues.

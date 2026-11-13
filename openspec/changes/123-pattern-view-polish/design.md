# Design

## Context

The current overview draws 16 rows in each visible bank and distributes leftover columns among cells. The step editor removes an identified row but does not insert into an empty cell. Saved pattern ticks already represent their playback positions. Changing the capture quantize setting does not move saved rows.

On 2026-11-13, the owner deferred the manual checks for change 122 and asked us to continue. Change 122 stays unarchived. This change has its own branch and PR.

## Decisions

### Keep the selected row in sight

Use the existing runtime empty-view toggle and its saved action ID. At entry, hide rows without visible hits, except the selected pad in the current bank. A bank with no such row stays in the fixed index but has no section unless it is current. `Alt+H` shows all eight bank sections and all 16 rows in each section. A bank with only hidden raw hits counts as empty. Recompute the viewport from the actual section heights so the selected bank remains visible.

### Fix step geometry

Each step uses three terminal cells. A one-cell separator marks each beat. The remaining columns stay blank at the right. Straight and triplet divisions both use their own whole-beat groups. Step numbers use the same three-cell positions as event marks. Keep beat shading, bar borders, cursor, and live playhead.

### Show velocity without changing event identity

Use three brightness levels for a visible hit: 1–42, 43–84, and 85–127. A cell with several rows uses the highest velocity for brightness and keeps its count mark. An off-grid single row keeps `~`. A selected event also shows its numeric velocity in step-edit text. Brightness adds detail. The glyph still identifies hit presence and multiplicity.

### Add only at an empty playback cell

Store the first tick of the selected view cell. Saved ticks already play at their stored position, independently of the current capture quantize setting. Keep the incoming raw MIDI velocity. Keyboard pads use their existing normal and Shift velocities. For a OneShot pad, store gate length zero. For a Gate pad, the default is one current view cell. An accepted insertion uses the same checked replacement, row identity, ordered publication, dirty state, and common Undo path as removal. Refuse an insertion at the 4096-row limit without changing A1.

An occupied cell keeps the current removal rule. The first pad press removes one event when the cell has one event. Several events require an explicit event choice. Repeats and releases do not edit or sound. A project or pattern change invalidates an old choice.

## Rejected options

- Do not spread leftover columns across steps. This makes the positions depend on the page width.
- Do not add a new MIDI action or change `pattern.overview_empty_toggle`. Existing learned bindings remain valid.
- Do not change format 5 or store edit cursor state. The editor is runtime state.

## Open questions

- Should a new Gate event last one view cell, until the edit pad releases, or act as a one-shot? Default: one view cell. The owner has not answered this choice yet.
- Should the three velocity levels use equal thirds? Default: 1–42, 43–84, and 85–127.

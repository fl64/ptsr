## MODIFIED Requirements

### Requirement: The assign mode ignores the rest
The system SHALL, while the assign mode is on, ignore every chop-screen action except the
pads, `Tab` (by hand / in order), `Enter` (apply), `Esc` (back), the slice pick — `J` `L`
(← →) step to the previous and the next slice and play it, the cursor goes to its start
as with the slice pick, the cuts stay — and the
undo: the cursor, zoom, cuts, the bounds, play and its mode, the equal parts and the region do not act
until the mode is left. An ignored key shows a hint in the status line: "assigning: Enter
applies, Esc leaves".

#### Scenario: No stray cut during the assign
- **WHEN** the assign mode is on and the owner presses `M`, `I` `K`, `Ctrl+J`,
  `Alt+L`, `Y` and `,`
- **THEN** nothing is cut, zoomed or moved: the cuts, the cursor, the view and the plan
  stay as they were, and the status line says how to apply or leave

#### Scenario: Pick the slice for a pad
- **WHEN** slice 5 is selected in the assign mode by hand, the owner presses `J` three times and hits pad C
- **THEN** slice 2 is planned on pad C; the cuts did not move

## ADDED Requirements

### Requirement: The region keeps the pads outside it
The system SHALL leave a pad that plays a slice falling outside a new region playing what it played: it gets its own bounds (`pad`) instead of moving onto a slice left in the region; the status names the pads that changed and undo links them back.

#### Scenario: A region over a laid-out break
- **WHEN** 9 slices lie on pads 1–9 and the owner sets the region start at the start of slice 9
- **THEN** pads 1–8 play what they played and say `pad`; pad 9 still follows its slice (now slice 1 of the region); undo brings the 9 slices and the links back

### Requirement: Undo keeps the slice under the cursor
The system SHALL, after an undo on the chop screen, select the slice under the cursor, so the slice line, the edit and the bounds act on the slice the cursor stands in.

#### Scenario: Edit after an undo
- **WHEN** the cursor stands in slice 9, the owner undoes a region and opens the slice edit
- **THEN** the edit is on slice 9

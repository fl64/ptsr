## MODIFIED Requirements

### Requirement: The assign mode ignores the rest

Assign SHALL ignore Chop actions except pads, `Tab` (by hand / in order), `Enter` (apply), `Esc` (back), slice selection, Undo and Redo.
`J` `L` (← →) SHALL select and play the previous or next slice and move the cursor to its start without changing cuts.
Ignored actions SHALL show the Apply and Back controls for the last input device. The ignored actions include loop, hear as pad and normalize.

#### Scenario: No stray cut during the assign
- **WHEN** the assign mode is on and the owner presses `M`, `I` `K`, `Ctrl+J`,
  `Alt+L`, `Y` and `,`
- **THEN** the cuts, the cursor, the view and the plan do not change
- **AND** the status line says how to apply or leave

#### Scenario: Pick the slice for a pad
- **WHEN** the owner selects slice 5 in Assign by hand, presses `J` three times, and hits pad C
- **THEN** the plan puts slice 2 on pad C
- **AND** the cuts do not move

#### Scenario: Assign ignores preview and gain changes
- **WHEN** a slice lies on a pad and the owner requests loop, hear as pad or normalize during Assign
- **THEN** the plan, playback settings, gain and Undo history stay the same
- **AND** the status line names Apply and Back

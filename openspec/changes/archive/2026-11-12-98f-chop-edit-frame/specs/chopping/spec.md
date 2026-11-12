# chopping Delta

## ADDED Requirements

### Requirement: The slice edit uses the pad edit frame

While the slice edit is open, the Chop right column SHALL show an edit frame titled `EDIT · BOUNDS` with the page lit, as the pad edit. A block line SHALL name the selected slice. Rows SHALL show `start`, `end`, `length` and `cursor` with their units. The selected bound SHALL be bright with `▸`. `length` and `cursor` SHALL be dim. Closing the edit SHALL restore the slice line.

#### Scenario: The slice edit uses the pad edit frame
- **WHEN** the owner opens the slice edit on slice 4 at 80x26 and 120x40, then presses K
- **THEN** the right column shows the frame `EDIT · BOUNDS` and `─ SLICE 4 ─`, with `start` bright first and `end` bright after K
- **AND** `length` and `cursor` are dim, and Esc restores the slice line

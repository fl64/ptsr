## ADDED Requirements

### Requirement: Redo restores a completed Undo

The system SHALL retain the state replaced by each successful Undo. Redo SHALL restore the latest undone edit in its applicable context and make it available to Undo again. The edit SHALL retain its original scope, state, and accepted sound. The owner SHALL see the changed item, a request to return to its context, or `nothing to redo`.

#### Scenario: Redo restores a grouped pad edit and its sound

- **WHEN** the owner undoes a grouped pad edit or a pad swap, and then redoes it
- **THEN** one Redo restores all the touched pad values, sounds, and pattern addresses
- **AND** the next Undo restores the previous values again

#### Scenario: Redo restores an accepted A1 take

- **WHEN** the owner undoes a completed A1 edit or finishes and undoes a live pass
- **THEN** Redo restores the accepted rows, order, and settings
- **AND** it waits until the pass and Undo are complete

#### Scenario: Redo restores Chop and Assign edits

- **WHEN** the owner undoes a Chop change, a normalization, a fresh Chop, or a target choice
- **THEN** Redo restores the edit and the applicable pad links or plan
- **AND** use from a different context preserves the pending edit

### Requirement: New edits end the redo history

A new undoable edit or a project change SHALL clear redo history. Playback, selection, and a refused edit SHALL preserve it.

#### Scenario: A new edit replaces the future steps

- **WHEN** the owner undoes an edit, plays or selects a pad, then makes a different edit
- **THEN** Redo remains available before the new edit
- **AND** the new edit clears Redo

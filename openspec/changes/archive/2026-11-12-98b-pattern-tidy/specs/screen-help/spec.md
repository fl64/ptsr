# screen-help Delta

## ADDED Requirements

### Requirement: Pattern tidy help and results

Pattern controls SHALL show a Clear A1 row with the count of A1 hits and the key that runs it. An empty A1 SHALL show the row dim. A result SHALL post a Done notice with its count, for example `A1: pad Q kick erased, 5 hits. Ctrl+Z restores.` or `A1 cleared: 12 hits.`. A command that changes nothing SHALL post an Info notice.

#### Scenario: Pattern tidy results name their counts
- **WHEN** the owner erases a pad with hits, clears A1, and then erases a pad without hits
- **THEN** each change posts a Done notice with its count, and the empty erase posts an Info notice
- **AND** pattern controls name the effective Clear A1 key

### Requirement: Erasure help names both forms

In whole-pad erasure, the help line SHALL say that a pad erases all its A1 hits. It SHALL name Undo and the exit keys. F1 SHALL explain both erasure forms: under the playhead in a pass, whole pads outside a pass.

#### Scenario: Erasure help names both forms
- **WHEN** the owner turns on erasure on Pads outside a pass and opens F1
- **THEN** the help line names the whole-pad erasure, Ctrl+Z and Esc
- **AND** F1 explains held erasure in a pass and whole-pad erasure outside a pass

#### Scenario: Erasure outside Pads names Pads
- **WHEN** no pass is active, the owner is on Chop and presses Alt+E
- **THEN** erasure stays off and an Info notice says to erase pads on Pads (F2)

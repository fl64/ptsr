# screen-help Delta

## ADDED Requirements

### Requirement: Counts and undo name what they mean

A count of one hit SHALL say `1 hit`. Other counts SHALL say `N hits`. This SHALL apply to pad counts, notices and questions. The undo notice of a pattern step SHALL name that step: `erase pad <key>`, `clear` or `pattern pass`.

#### Scenario: Counts and undo name what they mean
- **WHEN** the owner erases a pad with one hit, presses Ctrl+Z, clears A1 and presses Ctrl+Z
- **THEN** the pad count and notice say `1 hit`, and the undo notices say `undo: erase pad W on A1` and `undo: clear on A1`

### Requirement: The pad edit help comes first

While the pad edit is open, the help row SHALL show the edit group before the A1 group.

#### Scenario: The pad edit help comes first
- **WHEN** the owner opens the pad edit at 80x26
- **THEN** the help row shows the edit keys before the A1 keys

### Requirement: F1 wraps whole words

F1 SHALL wrap descriptions and keys at spaces. A word wider than its column SHALL wrap inside the word.

#### Scenario: F1 wraps whole words
- **WHEN** the owner opens F1 at 80x26
- **THEN** no line ends inside a word that fits its column

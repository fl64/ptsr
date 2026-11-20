## MODIFIED Requirements

### Requirement: Counts and undo name what they mean

A count of one hit SHALL say `1 hit`. Other counts SHALL say `N hits`. This SHALL apply to pad counts, notices and questions. The undo notice of a pattern step SHALL say what came back: `undo: pad <key> hits back on A1`, `undo: A1 hits back` or `undo: A1 before the pass`. A notice after a pattern step SHALL name the undo control of the last input device.

#### Scenario: Counts and undo name what they mean
- **WHEN** the owner erases a pad with one hit, presses Ctrl+Z, clears A1 and presses Ctrl+Z
- **THEN** the pad count and notice say `1 hit`, and the undo notices say `undo: pad W hits back on A1` and `undo: A1 hits back`

#### Scenario: The erase notice names the SMC-PAD undo
- **WHEN** the owner erases a pad with the SMC-PAD in erase mode
- **THEN** the notice names «PAD BANK + 2 undo», not Ctrl+Z

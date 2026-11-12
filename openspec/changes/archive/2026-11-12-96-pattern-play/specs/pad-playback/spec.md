# pad-playback Delta

## ADDED Requirements

### Requirement: Per-pad slice Pitch Gate

Each pad SHALL have a saved Pitch Gate boolean, true by default. It SHALL act only on sequenced notes whose onset uses a linked slice and positive pitch. Manual hits, Chop previews, whole-file sources, detached pad pieces and non-positive pitch SHALL NOT receive its cut. Changes SHALL affect future note eligibility, not deadlines of sounding notes.

#### Scenario: Pattern Pitch Gate eligibility preserves manual sound
- **WHEN** equivalent linked slices, whole files and detached pieces play manually and through A1 at negative, zero and positive pitch
- **THEN** only positively pitched linked slices with Pitch Gate On receive an automatic step cut
- **AND** manual hits and previews sound the same with the switch On or Off

#### Scenario: Pitch Gate changes apply to future note eligibility
- **WHEN** pitch, source or Pitch Gate changes after a note starts
- **THEN** that note retains its existing deadline and the next note uses the new eligibility

### Requirement: Pitch Gate uses the next grid boundary

An eligible note SHALL start its existing short cut at the first pattern grid line strictly after its compiled onset. Empty steps SHALL supply boundaries. Quantize Off SHALL use 1/16 for this gate. Strength SHALL NOT move grid lines. Straight and triplet grids SHALL retain their exact spacing. The loop end SHALL also be a boundary.

#### Scenario: Pitch Gate cuts at empty straight and triplet steps
- **WHEN** eligible notes play before empty steps on every supported straight or triplet grid
- **THEN** each cut starts within one output frame of the next grid line, even with no hit on that line
- **AND** Quantize Off uses 1/16 and changing strength does not move the gate grid

#### Scenario: Pitch Gate respects off-grid onsets and the loop end
- **WHEN** eligible hits lie on a line, between lines or just before the loop seam
- **THEN** each gate uses the strictly later line or the loop end
- **AND** seam releases precede new tick-zero notes without cutting those new notes

### Requirement: Pitch Gate preserves the source and cut path

Pitch Gate SHALL use the existing 5 ms short fade, not the pad's gate-release duration. The cut SHALL NOT move source bounds, extend playback, insert a universal attack or change mute groups. Natural source end SHALL remain authoritative. An earlier ordinary release SHALL start its envelope, but Pitch Gate SHALL still cap that fading voice. A short cut SHALL preserve current fade gain.

#### Scenario: Pitch Gate cuts without extending a slice or changing attack
- **WHEN** eligible slices end before or after their step boundary, including a gate pad with a long release
- **THEN** short slices end naturally and longer slices use the existing short fade at the boundary
- **AND** source bounds, attack, group membership and unrelated voices stay unchanged

#### Scenario: Pitch Gate caps an ordinary release tail
- **WHEN** an eligible gate note has its saved release before, on or after the Pitch Gate boundary
- **THEN** an earlier ordinary release starts its envelope and the Pitch Gate boundary still starts its short cut if that note remains
- **AND** coincident deadlines use the short cut without raising the current fade gain

### Requirement: Pitch Gate follows pad state and undo

Pitch Gate SHALL follow the existing pad change scope, save, swap, copy snapshot and undo rules. Reset to clean SHALL set it to true without changing the source. Clear SHALL retain it like other retained pad parameters. Cancelled or refused operations SHALL leave it unchanged. A scope toggle SHALL set every scope pad to the selected pad's inverse value in one undo step.

#### Scenario: Pitch Gate scope reset copy and undo preserve pad state
- **WHEN** keyboard and raw MIDI toggle mixed scope values, copy a snapshot, reset to clean, clear and undo
- **THEN** each operation follows the specified Pitch Gate state and existing source and undo rules
- **AND** unrelated pads and cancelled operations stay unchanged

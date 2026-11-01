## MODIFIED Requirements

### Requirement: Linked slices
The system SHALL keep the cuts of a file, also when none of its slices lies on a pad, and SHALL keep the pads with its slices linked to them, so a change of a cut changes these pads at once. A new cut inside a slice that lies on pads keeps these pads on the left part, the slice that still starts at their cut; the right part is a new slice on no pad.

#### Scenario: Fix a slice after the layout
- **WHEN** slices 1–8 of a break are on pads 1–8 and the owner opens pad 3 with F3 and moves the start of slice 3 by 20 ms to the right
- **THEN** pad 2 plays 20 ms longer and pad 3 starts 20 ms later, without laying the slices out again

#### Scenario: Open the whole break from a slice
- **WHEN** the owner selects a pad with slice 5 of a break and presses F3
- **THEN** the chop screen shows the whole break with all its cuts and slice 5 selected

#### Scenario: Join two slices
- **WHEN** pads 2 and 3 hold slices 2 and 3 and the owner joins slice 3 with slice 2
- **THEN** pads 2 and 3 both play the joined slice, and the pads with later slices keep playing the same sound under numbers one lower

#### Scenario: Chop again into equal parts
- **WHEN** pads hold slices of a break and the owner replaces its cuts with equal parts
- **THEN** these pads keep playing what they played and are no longer linked, the status says how many pads were unlinked

#### Scenario: Undo the new chop
- **WHEN** the owner has replaced the cuts with equal parts and presses undo
- **THEN** the old cuts are back and the pads are linked to them again

#### Scenario: Cut inside a slice on a pad
- **WHEN** slice 1 of a break lies on pad Q and the owner cuts inside slice 1
- **THEN** Q plays slice 1 from its start to the new cut and stays linked, the status says "pad 1-Q is shorter now; Ctrl+Z returns"; the right part is slice 2 on no pad; pads with later slices stay linked under numbers one higher

#### Scenario: The status says the pads changed
- **WHEN** slices are on pads and the owner moves, deletes or adds a cut or a region edge that bounds them
- **THEN** the status says how many pads changed and that Ctrl+Z returns them; after undo it says how many pads changed back

#### Scenario: Undo the cut
- **WHEN** the owner undoes the cut inside slice 1 of pad Q
- **THEN** the cut is gone and Q is linked to slice 1 again

#### Scenario: Restart
- **WHEN** the program is restarted
- **THEN** the cuts of each chopped file and the links of its pads are restored

#### Scenario: Chopped, nothing laid out
- **WHEN** the owner cuts a sample, puts no slice on a pad and restarts the program
- **THEN** the chop screen of that sample shows the same cuts

### Requirement: Listen mode
The system SHALL, while the chop screen is in the listen mode, show the real pads of the current bank on the grid, as the pads screen does, and play them on a hit; a pad that plays a slice of this file says so (`slice 3`), the other pads are dimmed. The chop screen opens on the bank that holds slices of this file when the current bank holds none; the bank actions turn the banks. Slices on no pad are heard with the cursor (the arrows, the jump by cuts). Until a slice of this file lies on a pad, the grid is a keyboard of slices instead: key k plays and selects slice 16·b + k, where b is the bank picked with the bank actions, the row over the grid says `slices 1–16`, and every cell draws its slice's wave.

#### Scenario: Before the layout
- **WHEN** a sample is cut into 8 slices, none of them lies on a pad, and the owner hits the third pad key on the chop screen
- **THEN** slice 3 plays and is selected, the grid names the cells `slice 1`…`slice 8`, and the row over it says `slices 1–8`

#### Scenario: Check a slice
- **WHEN** slices 1–8 of a break lie on pads 1–8 of bank 2, bank 1 is shown, the owner opens the break on the chop screen from the files screen and hits the third pad key
- **THEN** the grid shows bank 2, the third pad plays slice 3 from its start to its end without a click, and the dimmed pads of bank 2 play what they hold

### Requirement: Pad hit selects the slice
The system SHALL, in the listen mode, make the slice of this file played by a pad hit the selected slice, so the next edit acts on it; a hit on a pad that plays no slice of this file selects nothing.

#### Scenario: Fix the slice just heard
- **WHEN** slice 6 lies on pad 6 of the bank shown and the owner hits it and then moves the slice start
- **THEN** slice 6 is selected and highlighted, and its start moves

### Requirement: Where a slice lies
The system SHALL name on the slice line of the chop screen the pads that hold the selected slice, as bank and key joined by a dash (`2-4`, `2-Q`) like the labels at the slice numbers (27), also when the slice is too narrow for its label.

#### Scenario: Two pads hold the slice
- **WHEN** slice 3 lies on pad Q of the current bank and pad W of bank 2 and the owner selects slice 3
- **THEN** the slice line says "on 1-Q, 2-W"

## ADDED Requirements

### Requirement: Slices on pads are marked
The system SHALL mark in the row over the wave where every slice lies: a slice on a pad gets a plate in the color of the bank of its first pad across the slice's width, with the slice number and its pads on it (`5 2-Q`); a slice on no pad shows its number with a dim `○` and no plate. The bank row of the chop screen is shown in the listen mode too and marks the banks that hold slices of this file. While slices with `○` stand next to slices on pads, the hint says how to place them.

#### Scenario: A cut leaves a slice on no pad
- **WHEN** slices 1–8 lie on pads and the owner cuts inside slice 5
- **THEN** slice 5 wears a plate in the color of its bank saying `5 1-A`, the new slice 6 shows a dim `○` and no plate, and the hint says "○ slices lie on no pad: P, J L and a pad hit place them"

## REMOVED Requirements

### Requirement: Pages of slices
**Reason**: the grid of the listen mode shows the real pads (36); slices on no pad are heard with the cursor, `Ctrl+1`…`8` turn the banks.
**Migration**: none; `PgUp` `PgDn` keep moving the cursor between the cuts.

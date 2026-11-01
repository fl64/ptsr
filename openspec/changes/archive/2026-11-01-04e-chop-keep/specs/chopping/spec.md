## MODIFIED Requirements

### Requirement: Assign slice by slice
The system SHALL open the assign mode with an empty plan made by hand and the slice selected before it opened; each pad hit puts the selected slice on that pad and selects the next slice not yet planned, until all slices are planned.

#### Scenario: Tap pads in turn
- **WHEN** a sample is cut into 4 slices, slice 1 is selected, the owner presses P (was Enter before 21) and hits pads Q, 1, W, E
- **THEN** slice 1 is planned on Q, slice 2 on 1, slice 3 on W, slice 4 on E, and the status says that all 4 slices are placed and Enter applies

#### Scenario: Assign the slice just heard
- **WHEN** the owner hits the fifth pad key in the listen mode, presses P (was Enter before 21), hits pad Q and presses Enter again
- **THEN** slice 5 lies on Q and no other pad changed

#### Scenario: All placed
- **WHEN** all slices are planned and the owner hits another pad
- **THEN** nothing is planned there, the pad plays what is planned for it if anything

#### Scenario: Undo a tap
- **WHEN** the owner undoes the last tap
- **THEN** that slice is no longer planned and is selected again

### Requirement: Linked slices
The system SHALL keep the cuts of a file, also when none of its slices lies on a pad, and SHALL keep the pads with its slices linked to them, so a change of a cut changes these pads at once. A new cut inside a slice that lies on pads SHALL unlink these pads instead: they keep playing exactly what they played.

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
- **THEN** Q plays the same piece as before, its panel says `amen.wav · pad` (27), the status says that pad Q no longer follows the cuts; pads with other slices stay linked

#### Scenario: The status says the pads changed
- **WHEN** slices are on pads and the owner moves, deletes or adds a cut or a region edge that bounds them
- **THEN** the status says how many pads changed and that Ctrl+Z returns them; after undo it says how many pads changed back

#### Scenario: Undo the cut
- **WHEN** the owner undoes the cut that unlinked pad Q
- **THEN** the cut is gone and Q is linked to slice 1 again

#### Scenario: Restart
- **WHEN** the program is restarted
- **THEN** the cuts of each chopped file and the links of its pads are restored

#### Scenario: Chopped, nothing laid out
- **WHEN** the owner cuts a sample, puts no slice on a pad and restarts the program
- **THEN** the chop screen of that sample shows the same cuts

## ADDED Requirements

### Requirement: Where a slice lies
The system SHALL name on the slice line of the chop screen the pads that hold the selected slice, as bank and key like the labels at the slice numbers (27), also when the slice is too narrow for its label.

#### Scenario: Two pads hold the slice
- **WHEN** slice 3 lies on pad Q of the current bank and pad W of bank 2 and the owner selects slice 3
- **THEN** the slice line says "on 1Q, 2W"

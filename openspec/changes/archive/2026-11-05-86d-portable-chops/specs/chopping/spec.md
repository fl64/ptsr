## ADDED Requirements

### Requirement: Portable chop state

Moving a project folder SHALL preserve every saved chop's region, cuts, cut IDs, epoch and next ID.
This SHALL include cuts with no assigned pad inside an older pad-linked variant.
F3 on a linked pad SHALL open its saved variant.
A bound edit SHALL affect only that variant's linked pads.
Pad sound settings and restored slice sound SHALL remain unchanged for keyboard and controller input.

#### Scenario: Moving keeps unassigned cuts of older chops
- **WHEN** several pad-linked variants contain extra cuts with no assigned pad and the project folder moves
- **THEN** opening a pad from each variant restores every saved cut and its original identity
- **AND** those cuts survive saving and opening the moved project again

#### Scenario: Editing a moved chop changes only its linked pads
- **WHEN** keyboard or raw MIDI input moves a bound in one restored variant
- **THEN** only pads linked to that variant follow the bound
- **AND** the cuts, links and sound settings of the other variants stay unchanged

#### Scenario: Moved slices keep their sound
- **WHEN** restored slice pads render before and after project relocation with matching input, velocity and engine state
- **THEN** each matching output agrees sample for sample
- **AND** automatic checks find no non-finite samples or clipping and check expected silence and hit timing within one sample

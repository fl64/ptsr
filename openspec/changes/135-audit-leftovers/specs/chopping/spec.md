## ADDED Requirements

### Requirement: Join a slice with its neighbour

In the slice edit, the panel SHALL show `join left` and `join right`. `join left` SHALL remove the left bound of the selected slice. `join right` SHALL remove its right bound. While the panel shows these cells, the wave SHALL mark both bounds of the selected slice. A cell without a neighbour slice SHALL be dim, and a press SHALL show a notice that says why. After a join, a notice SHALL name the joined slices.

#### Scenario: Join left names the joined slices
- **WHEN** the owner selects slice 3 of 4, opens the slice edit and presses `join left`
- **THEN** a notice says «slices 2 and 3 joined» and the chop has 3 slices

#### Scenario: The slice edit marks the bounds to join
- **WHEN** the owner selects slice 3 of 4 and holds PAD BANK in the slice edit
- **THEN** the panel shows `join left` and `join right`, and the wave marks the left and the right bound of slice 3

#### Scenario: The first slice has no left neighbour
- **WHEN** the owner selects slice 1 of 4 in the slice edit and presses `join left`
- **THEN** the cell is dim, the chop keeps 4 slices and a notice says «slice 1 has no slice on the left»

#### Scenario: One slice has nothing to join
- **WHEN** the chop has one slice and the owner presses `join right` in the slice edit
- **THEN** both cells are dim, the chop keeps one slice and a notice says «nothing to join: one slice»

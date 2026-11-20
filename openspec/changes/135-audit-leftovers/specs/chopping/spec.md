## ADDED Requirements

### Requirement: A cut removal names its bound and its result

In the slice edit, the hint SHALL name the bound that each removal control removes. `cut -` removes the left bound of the selected slice. `end -` removes its right bound. After a removal, a notice SHALL name the slices that joined.

#### Scenario: The slice edit names the bounds to remove
- **WHEN** the owner selects slice 3 of 4 and opens the slice edit
- **THEN** the hint names the left bound of slice 3 for `cut -` and its right bound for `end -`

#### Scenario: A removed cut names the joined slices
- **WHEN** the owner selects slice 3 of 4, opens the slice edit and presses `cut -`
- **THEN** a notice says «slices 2 and 3 joined» and the chop has 3 slices

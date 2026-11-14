## RENAMED Scenarios

- FROM: `#### Scenario: Parts of the region`
- TO: `#### Scenario: Region from I to O`

- FROM: `#### Scenario: Mark and cut`
- TO: `#### Scenario: Region from I to O`

- FROM: `#### Scenario: Clear`
- TO: `#### Scenario: Clear and undo`

- FROM: `#### Scenario: Zoom around the cursor`
- TO: `#### Scenario: Zoom in and out`

## REMOVED Scenarios

#### Scenario: Both paths are named
**Reason**: Change 41 changed `Enter` from direct placement to taking a file in hand. The old Files hint is no longer correct. `Chop a found break` and `See where the file goes` cover the current paths.

#### Scenario: Limits
**Reason**: Change 21 changed the zoom limit to a minimum view of 256 frames, independent of the chop region. `Zoom in and out` covers the current view behavior.

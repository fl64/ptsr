# screen-help Delta

## ADDED Requirements

### Requirement: Quantize help names the next hits

When the owner selects Quantize or Strength in pattern controls, the row hint SHALL start with `next hits`. Strength SHALL keep its step after it: `next hits, step 10`. F1 SHALL say that Quantize and Strength set the next recorded hits and do not move saved hits.

#### Scenario: Quantize help names the next hits
- **WHEN** the owner selects Quantize, then Strength in pattern controls, then opens F1
- **THEN** each selected row shows `next hits`
- **AND** F1 says that a change does not move saved hits

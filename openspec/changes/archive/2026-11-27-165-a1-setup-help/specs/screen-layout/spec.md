## MODIFIED Requirements

### Requirement: Pattern labels and hints stay whole

The A1 strip SHALL show the pad row label `pad` with its key whole at 80x26 and 120x40. In pattern controls, a row hint that does not fit beside its value SHALL take its own line under the value. If Enter is bound, the key line SHALL name close on a value row and clear on the clear row.

#### Scenario: Pattern labels and hints stay whole
- **WHEN** the owner selects pad Q with A1 hits, opens pattern controls and selects quantize, strength and clear A1 at 80x26 and 120x40
- **THEN** the strip shows `pad Q`, and `next hits`, `next hits, step 10` and `Enter` show whole
- **AND** the key line names close on value rows and clear on the clear row

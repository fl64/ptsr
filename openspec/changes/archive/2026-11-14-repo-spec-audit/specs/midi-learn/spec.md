## RENAMED Scenarios

- FROM: `#### Scenario: A command`
- TO: `#### Scenario: A command by its key`

- FROM: `#### Scenario: A control asks first`
- TO: `#### Scenario: A turn waits for Enter`

- FROM: `#### Scenario: Clear asks too`
- TO: `#### Scenario: Clear waits for Enter`

## REMOVED Scenarios

#### Scenario: Pad hits in the list
**Reason**: Change 28 replaced individual pad actions in the F12 list with the `pads 1–16` row. `Sixteen pads in a row` covers that row.

#### Scenario: Learn a pad hit
**Reason**: Change 28 replaced individual pad binding with a sequential pad run. `Sixteen pads in a row` covers the current flow.

#### Scenario: A knob onto the volume
**Reason**: Changes 52 and 57 replaced the F4 walk and immediate binding with F10 selection and pending confirmation. `Do, then F10` covers the current flow.

#### Scenario: Controller pads onto the grid
**Reason**: Change 57 stopped immediate binding of controller pads. `A turn waits for Enter` covers the current flow.

#### Scenario: Clear
**Reason**: Change 52 made clearing ask first. Change 57 made `none` pending until `Enter`. `Clear waits for Enter` covers the current flow.

#### Scenario: A free control binds and moves on
**Reason**: Change 57 stopped immediate binding. `A turn waits for Enter` covers the current flow.

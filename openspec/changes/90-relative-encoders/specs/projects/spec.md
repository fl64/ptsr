# projects Delta

## ADDED Requirements

### Requirement: A relative knob selects a project

While the project picker is open, a knob bound to `projects.scroll` SHALL move its selection by signed steps and clamp at the ends. It SHALL not open, create or change a project until the owner confirms the selected row. It SHALL leave pending source work and the current project intact. Outside the picker, this target SHALL do nothing.

#### Scenario: Relative project selection waits for confirmation
- **WHEN** the owner opens the project picker and turns a relative knob through its rows
- **THEN** the selected row changes and the open project stays active
- **AND** the existing confirmation action opens only the row that the owner selected

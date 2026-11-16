# sample-loading Delta

## ADDED Requirements

### Requirement: A relative knob selects files

On Files, a knob bound to `files.scroll` SHALL move the selected row by its signed steps and clamp at the ends. It SHALL use the existing preview behavior after selection changes. Outside Files, this target SHALL do nothing. The keyboard list keys SHALL keep their roles.

#### Scenario: Relative file selection previews the last row
- **WHEN** Files shows several sounds and a relative knob moves past them in both directions
- **THEN** the selection stops at the list ends and the preview follows the final selected file
- **AND** skipped previews do not start after the final one

## ADDED Requirements

### Requirement: The pad panel in blocks
The system SHALL group the parameter rows of the pad panel by page, in the page order
TONE, ENV, FILTER, SOURCE: every block stands under a thin line with the page name, the
line of the page the edit is on is lit. The edit walks the pages in the same order with
`Tab` and `I` `K`, and the frame title names them in it.

#### Scenario: Blocks
- **WHEN** the pad edit is open
- **THEN** the panel shows the lines TONE, ENV, FILTER, SOURCE in this order, each over its rows

#### Scenario: SOURCE last
- **WHEN** the edit is on the reso of the FILTER page and the owner presses `Tab`
- **THEN** the source is picked, on the SOURCE page; one more `Tab` after the end goes back to the volume

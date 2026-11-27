## ADDED Requirements

### Requirement: Full Help remains clear with the command panel open

At 80x26 and 120x40, the command grid SHALL stay behind all Full Help tabs. The Full Help border SHALL name the available close, tab and navigation controls. The SMC-PAD hint SHALL follow the hold or second bank route. A question SHALL keep priority and show its own answers. When Help closes, the previous screen and panel SHALL reappear.

#### Scenario: Panel and Full Help stay readable together
- **WHEN** the owner opens Full Help with the command panel open and visits Help, Messages and Controller
- **THEN** each tab shows its full text area and the border shows close and navigation
- **AND** after close, the previous panel and work screen reappear

#### Scenario: The Help border names the route for the current pad bank setting
- **WHEN** the owner uses SMC-PAD in Full Help with the hold on or off
- **THEN** the help border names the available hold or second bank route

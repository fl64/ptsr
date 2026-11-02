## ADDED Requirements

### Requirement: Pad pages exclude shared effects

The pads screen SHALL show only TONE, ENV, FILTER, SOURCE and FX in its parameter page
ring. The FX page contains the pad drive and sends. Shared FX 1 and FX 2 SHALL be
controlled on the separate shared FX screen, not from the pad edit.

#### Scenario: The pad page ring stops at sends
- **WHEN** the pad edit is on TONE and the owner presses Shift+Tab
- **THEN** the pad FX page is selected, not a shared effect

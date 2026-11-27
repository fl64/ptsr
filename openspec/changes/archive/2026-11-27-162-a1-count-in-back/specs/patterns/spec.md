## MODIFIED Requirements

### Requirement: Cancel count in without recording

A second recording toggle or an unclaimed Esc SHALL cancel count-in and leave transport stopped. On the A1 overview, Esc and the SMC-PAD command P1 SHALL cancel count-in and keep the overview, bank and selected pad. Help and visible questions SHALL receive Back first. A newly created empty A1 SHALL remain. Cancellation SHALL add no recorded rows or pass Undo step. Releases from warm-up pads SHALL retain their original sound ownership.

#### Scenario: Cancel count in with a pad held
- **WHEN** a pad sounds during count-in and the owner cancels before tick zero and releases that pad
- **THEN** no recorded hit or pass Undo step appears and transport stays stopped
- **AND** the release reaches its original pad without recording a late event

#### Scenario: Back cancels count-in on A1
- **WHEN** the owner presses Esc or command P1 during count-in on the A1 overview
- **THEN** count-in stops, the transport stays stopped and the overview stays open at the same bank and pad
- **AND** if Help or a question covers A1, the first Back closes that window and count-in continues

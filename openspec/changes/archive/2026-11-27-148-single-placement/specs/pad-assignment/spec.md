## ADDED Requirements

### Requirement: The hand holds one placement item

A successful pickup SHALL replace the item in hand. An item is a copy, a swap
source, a Files sample or a saved recording. A refused pickup SHALL preserve the
previous item. A swap source SHALL have an assigned file and SHALL be ready.
Both input devices SHALL put the current item on the target.

#### Scenario: Successful pickups replace the previous placement
- **WHEN** the owner takes a copy and then a swap, or a swap and then a copy
- **THEN** only the last item stays in hand
- **AND** a keyboard or MIDI pad hit applies only that item
- **AND** the next hit plays and Undo restores the placement

#### Scenario: File pickup replaces the pad operation
- **WHEN** the owner takes a file while a pad copy or swap waits
- **THEN** the file takes its place in hand
- **AND** a keyboard or MIDI pad hit puts the file on the target

#### Scenario: A refused pickup preserves the held item
- **WHEN** an empty, missing or loading pad cannot give the requested copy or swap
- **THEN** the app explains the refusal and keeps the item in hand

#### Scenario: A saved recording replaces the held item
- **WHEN** the app saves a recording while a copy or swap waits
- **THEN** the recording replaces the item in hand
- **AND** a question about the previous item closes

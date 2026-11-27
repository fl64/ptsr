## ADDED Requirements

### Requirement: A held Gate can end in the folder picker

The folder picker SHALL pass the release of a keyboard Gate it did not start. The release SHALL use the original pad address. A new keyboard pad press in the picker SHALL stay silent.

#### Scenario: A folder picker lets an earlier Gate end

- **WHEN** a key starts a Gate and the owner changes banks and opens a folder picker before releasing it
- **THEN** the original Gate stops
- **AND** pressing and releasing another pad key in the picker does not play or release a pad

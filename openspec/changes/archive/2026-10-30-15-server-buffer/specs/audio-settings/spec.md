## MODIFIED Requirements

### Requirement: Buffer size on the fly
The system SHALL take the sound server's buffer size as it is at start (the default, `buffer_size = 0`), let the owner change the buffer size (64, 128, 256, 512, 1024, auto) on the settings screen without a restart, and show the actual size and the latency in the status line. With auto chosen, the program never asks the server for another size.

#### Scenario: Start does not touch the graph
- **WHEN** the config has `buffer_size = 0` (or no line) and the server runs 1024
- **THEN** the program starts at 1024 without asking the server to change anything, and other programs' sound keeps playing without a hiccup

#### Scenario: An explicit size still applies
- **WHEN** the config has `buffer_size = 256`
- **THEN** the program asks the server for 256 at start, as before

#### Scenario: Server did not accept the buffer
- **WHEN** the sound server did not accept the selected size
- **THEN** the status line shows the actual size, a message at the bottom says the size was not accepted

#### Scenario: The buffer row explains itself
- **WHEN** the owner stands on the buffer row of the settings screen
- **THEN** it shows auto with the actual size in parentheses and one line saying what it is for: follow the server by default, go smaller only if pads feel late, bigger if clicks are heard

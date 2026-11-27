## MODIFIED Requirements

### Requirement: Learn the A1 overview screen action

F12 MIDI SHALL list `screen.pattern` as a GLOBAL button action. A learned note or CC button SHALL open the same Pattern screen as F5. Existing confirmation, clearing, persistence and layer precedence SHALL apply. Learning SHALL NOT open the Pattern screen or make a pad sound. No factory profile binding SHALL be added.

#### Scenario: Learned overview button follows screen and learning rules
- **WHEN** the owner learns `screen.pattern` through a raw MIDI note or CC, confirms it, saves and reloads
- **THEN** a button press opens the Pattern screen as F5 does, without changing existing bindings
- **AND** selection, cancellation and release during learning do not open the Pattern screen or play a pad

### Requirement: Learn the empty-bank switch

F12 MIDI SHALL list `pattern.overview_empty_toggle` as a button action. A learned note or CC button SHALL add or remove the empty rows of the selected bank only while the Pattern screen is open. The action SHALL keep the ordinary confirmation, clearing and persistence rules. No factory profile binding SHALL be added.

#### Scenario: Learned empty-bank button changes only visibility
- **WHEN** the owner learns the empty-bank action and sends a raw MIDI button press on and off the Pattern screen
- **THEN** only the press on the Pattern screen changes the shown rows
- **AND** the pattern, pad modes and the current bank stay unchanged

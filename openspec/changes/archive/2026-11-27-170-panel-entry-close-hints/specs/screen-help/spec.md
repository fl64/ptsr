## ADDED Requirements

### Requirement: The MIDI footer names the available panel route

The idle MIDI footer SHALL name pad bank 2 if the PAD BANK hold shortcut is off or unavailable. In Pad Edit and Slice Edit, the MIDI mode line SHALL name two P1 presses in pad bank 2 when the panel is hidden. It SHALL name one P1 press when the panel is visible. A verified hold shortcut SHALL keep its hold hint. The keyboard mode line SHALL keep its close key.

#### Scenario: Panel entry and editor close follow the available route
- **WHEN** the owner uses a connected SMC-PAD without a working hold shortcut
- **THEN** the idle footer names pad bank 2
- **AND** in Pad Edit or Slice Edit, the first P1 shows the panel and changes the close hint to one P1
- **AND** the next P1 closes the editor on the same work screen

## MODIFIED Requirements

### Requirement: Panel replaces the pad grid without moving the frame

Outside narrow A1, the panel SHALL use the pad-grid area. It SHALL keep the title, strip, bank row, right column, three-row help and status in place. Cells SHALL follow first-bank pad order. Labels SHALL fit eight cells per line. The panel SHALL stay readable with kitty graphics on or off. It SHALL keep questions, text entry and warnings visible. Full Help SHALL use its usual area. Its border SHALL show the Help panel controls.

#### Scenario: Panel grid fits both supported sizes
- **WHEN** the panel shows its main and context pages at 80 by 26 and 120 by 40
- **THEN** all 16 cells, page title, fixed bottom row and safety indicators fit inside their existing areas
- **AND** pad-wave images cover no cell and leaving the panel restores the current wave

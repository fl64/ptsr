## ADDED Requirements

### Requirement: A1 overview shows every bank in one vertical list

The A1 overview SHALL arrange all eight banks in one vertical list. Each shown bank SHALL keep its 16 pad rows in pad order. Each row SHALL show the pad key, the pad name and the visible events at the selected view division. The view SHALL use compiled playback ticks without changing saved raw hits. It SHALL keep hidden raw hits hidden and show visible seam hits at the loop start.

#### Scenario: Overview shows all pad rows and source events
- **WHEN** A1 has hits on several pads and banks, including off-grid hits and two source hits in one cell
- **THEN** each shown bank keeps all 16 pad rows in pad order, including empty rows
- **AND** each occupied cell shows its event type and count without merging source hits or changing the take

#### Scenario: Overview filters hidden hits before seam wrapping
- **WHEN** A1 has a hidden row beyond its shortened length and a visible row that quantizes to the seam
- **THEN** the hidden row does not appear in any overview cell
- **AND** the visible seam row appears once at the loop start without changing the raw rows

### Requirement: Overview follows the shared view division and bank

The overview SHALL use the same runtime view division as the Pads strip. Existing bank controls SHALL move the viewport to the selected bank section. Changing the division or current bank SHALL change only the overview projection and selection. It SHALL NOT change playback, quantize, raw hits, history or dirty state. The view division SHALL return to 1/16 after a project change, as on Pads.

#### Scenario: Overview bank and view changes leave A1 unchanged
- **WHEN** the owner switches banks and view divisions while A1 plays
- **THEN** the viewport moves to each selected bank section with its 16 pads and the selected division, and A1 keeps its timing
- **AND** raw rows, quantize, history and dirty state stay unchanged

### Requirement: Overview can hide empty bank sections

The overview SHALL show all eight bank numbers in a fixed index. The index SHALL show each bank with visible A1 hits. By default, the vertical list SHALL show all eight bank sections. The empty-bank control SHALL hide or show sections with no visible A1 hits without changing A1 or pad selection. The current bank SHALL remain visible. A bank with only hidden raw hits SHALL count as empty in this view.

#### Scenario: Overview hides empty banks without losing access
- **WHEN** A1 has visible hits in two banks and hidden raw hits in another bank
- **THEN** the fixed index and list show all eight banks at entry
- **AND** the empty-bank control hides other sections but keeps the active and current banks without changing A1

#### Scenario: Overview bank navigation reaches a hidden empty bank
- **WHEN** the owner selects a bank with no visible A1 hits while empty banks are hidden
- **THEN** that bank section appears with 16 empty rows and the bank index marks it as current
- **AND** another bank selection restores the ordinary hidden-empty view

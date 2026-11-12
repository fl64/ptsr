# midi-learn Delta

## ADDED Requirements

### Requirement: Learn pattern tidy commands

F12 MIDI SHALL list `pattern.clear` as a GLOBAL button action. A learned button SHALL run the same App action as the keyboard path and SHALL still ask first. A learned `pattern.erase` button SHALL toggle whole-pad erasure on Pads outside a pass, as Alt+E does. In that mode, MIDI pad notes SHALL erase whole pads as keyboard pads do. No factory device binding SHALL be introduced.

#### Scenario: Learned erase and clear buttons match the keys
- **WHEN** learned CC buttons toggle erasure on Pads and run Clear A1 through raw MIDI profile routing
- **AND** a MIDI pad note arrives in erasure
- **THEN** the note erases its whole pad without sound, as a keyboard pad does
- **AND** Clear A1 asks before it clears, and each result is one Undo step

### Requirement: Knobs turn pattern controls while they are open

While pattern controls are open, edit knobs 1, 2 and 3 SHALL turn Bars, Quantize and Strength. Other learned knobs that change a pad, a slice or an effect SHALL change nothing. Master volume, tempo and the learned pattern knobs SHALL keep their roles. MIDI pad notes SHALL still play pads.

#### Scenario: Knobs turn A1 while pattern controls are open
- **WHEN** pattern controls are open on Pads and raw MIDI turns edit knobs 1–3 and a learned pad volume knob
- **THEN** Bars, Quantize and Strength change, and the pad volume stays unchanged
- **AND** a MIDI pad note plays its pad

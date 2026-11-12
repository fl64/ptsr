# keyboard Delta

## ADDED Requirements

### Requirement: Pitch Gate keyboard toggle

Shipped Alt+G on Pads SHALL run `pad.pitch_gate_toggle` with edit open or closed. It SHALL toggle once per press, not on repeat or release. It SHALL work in English and Russian layouts with extended and legacy keyboard input. Existing pad keys, pages, knobs, transport and preview bindings SHALL remain unchanged. Other screens SHALL gain no shipped Alt+G action.

#### Scenario: Pitch Gate keyboard toggle works in both layouts
- **WHEN** Pads receives Alt+G presses, repeats and releases in both supported layouts and input forms, with edit open or closed
- **THEN** each press changes Pitch Gate once for the existing scope and one undo restores that scope
- **AND** repeats and releases do not toggle it again or play a pad

### Requirement: Pitch Gate keys preserve input priorities

Text, questions, pickers and learning SHALL retain their existing input priorities. F10 SHALL select the effective Pitch Gate command without executing it. Owner keymap entries, including `none`, SHALL win without file rewriting. A hidden shipped Alt+G SHALL use the existing startup warning queue. Hints and F1 SHALL use effective bindings.

#### Scenario: Pitch Gate input cannot bypass overlays or owner keys
- **WHEN** Alt+G reaches text, a question, picker or learning, or an owner entry changes or removes its shipped action
- **THEN** existing input priority or that owner entry wins without an underlying Pitch Gate toggle
- **AND** applicable startup warnings name the hidden default without rewriting the keymap

#### Scenario: Pitch Gate help and screen learn follow effective keys
- **WHEN** the owner rebinds Pitch Gate on Pads, opens help or selects its command in F10
- **THEN** help names the effective key and screen learn selects the command without executing it
- **AND** existing EDIT-only bottom hints keep their layout with the command available in F1

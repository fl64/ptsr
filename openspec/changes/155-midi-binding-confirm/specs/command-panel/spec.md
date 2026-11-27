## MODIFIED Requirements

### Requirement: Visible questions answer with pads 1 and 4

While a question is visible, device pad 1 SHALL cancel and device pad 4 SHALL confirm on press except for binding questions. A binding question SHALL show the keyboard confirm and give device pad 1 to cancel. This SHALL apply with owner bindings and a hidden panel. Both pads SHALL answer from either note bank without a hold. Other pads SHALL stay silent. PLAY and STOP SHALL never answer and MAY keep their transport roles. A release SHALL NOT answer. Keyboard answers SHALL keep their rules.

#### Scenario: Pad 1 and pad 4 answer a visible question on the first hit
- **WHEN** a question accepts controller confirmation and the owner presses pad 1 or 4 from a hidden panel in separate runs
- **THEN** pad 1 cancels and pad 4 confirms through the normal question action
- **AND** neither press first reveals the panel or acts as an ordinary cell

#### Scenario: A question takes pads 1 and 4 without a hold
- **WHEN** a question that accepts controller confirmation is visible, the hold shortcut is on and PAD BANK is not held
- **THEN** device pads 1 and 4 from either note bank answer the question
- **AND** a hit on another pad stays silent and the question stays

#### Scenario: A panel action followed by pad 4 confirms only the question
- **WHEN** a destructive panel cell opens a question that accepts controller confirmation and device pad 4 arrives before the next redraw
- **THEN** pad 4 confirms only that visible question once
- **AND** no earlier cell action repeats and no second target changes

#### Scenario: PLAY and STOP do not answer a pad 1 and pad 4 question
- **WHEN** a question is visible and PLAY or STOP sends a tap
- **THEN** the context playback action runs where available
- **AND** the question and its target stay unchanged

#### Scenario: A held pad 4 release does not answer
- **WHEN** the owner holds device pad 4 as a Gate pad, a question opens and the owner releases the pad
- **THEN** the Gate sound ends once and the question stays open

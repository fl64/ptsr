## MODIFIED Requirements

### Requirement: Pad grid and banks
The system SHALL provide 16 pads in a 4×4 grid and 8 banks numbered 1–8; the bank is switched with a command from the keyboard or a controller: next/previous, or directly with Ctrl+1…8. On screen a bank is always shown with the word "bank" so it is not confused with the pad keys 1–4. Each bank in the bank row also shows its fill: two braille glyphs whose 4×4 dots mirror the pads of that bank, a lit dot for a pad with a sound; the fill is shown on every screen that shows the bank row and changes at once when a pad gets or loses a sound.

#### Scenario: Bank change
- **WHEN** the owner runs "next bank" on bank 1
- **THEN** bank 2 becomes active, the screen shows the pads of bank 2, sounding pads of bank 1 play to the end

#### Scenario: Direct bank choice
- **WHEN** the owner presses Ctrl+3
- **THEN** bank 3 becomes active; the plain 3 key still plays pad 3

#### Scenario: See what a bank holds
- **WHEN** bank 1 has sounds on pads 1 and Q and the other banks are empty
- **THEN** the bank row shows two lit dots in the glyphs of bank 1 and no lit dots in the glyphs of the other banks

#### Scenario: Fill follows a change
- **WHEN** a sound is put on pad A of bank 5 or removed from it
- **THEN** the glyph of bank 5 in the bank row lights that dot or goes dark at once, on every screen that shows the bank row

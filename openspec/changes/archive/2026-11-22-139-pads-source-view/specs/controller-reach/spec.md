## ADDED Requirements

### Requirement: Sound and Rhythm strip controls

On Pads Alt+9 and P16 in its visible controller panel SHALL switch Sound and Rhythm. Alt+0 and P16 in the Pad Edit panel SHALL toggle the sample pin. The pin SHALL work only with a loaded selection or an active pin. A visible hint SHALL name the switch. The panel SHALL show the current view and the pin state.

#### Scenario: Keyboard and SMC-PAD switch and pin the Pads strip

- **WHEN** the owner uses the keyboard or raw SMC-PAD messages to switch the strip and pin a sample
- **THEN** each route gives the same view and pin state, and the hints fit at 80x26 and 120x40

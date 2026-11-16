# keyboard Delta

## ADDED Requirements

### Requirement: Keyboard reaches SMC-PAD context transport actions

The shipped keymap SHALL bind `Alt+T` to `play.context` and `Alt+Y` to `stop.context` on the right half of the keyboard. They SHALL use the same screen and question behavior as the SMC-PAD PLAY and STOP taps. Space SHALL keep its existing transport or preview toggle on each screen. Owner key overrides SHALL keep their current priority.

#### Scenario: Context PLAY and STOP match keyboard and raw MIDI
- **WHEN** separate runs send `Alt+T` and `Alt+Y` or the corresponding raw SMC-PAD button taps on Pads, Files and Chop
- **THEN** each run gets the same start-only and stop-only result for its current screen
- **AND** Space and existing keyboard pad keys keep their current roles

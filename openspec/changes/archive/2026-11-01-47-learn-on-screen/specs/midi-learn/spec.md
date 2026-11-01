## ADDED Requirements

### Requirement: Learn on the screen
The system SHALL turn a learn mode on and off with `F4` on the pads, chop and files
screens. In the mode every element of the screen that can take a control carries a label
with its control, or `---` when it has none: the pads of the grid, the bank switch, the
parameters of the panel, the commands of the help line. One element is picked and blinks;
a control moved on the controller is bound to it at once, saved as a learned assignment of
the learn list (the F12 MIDI tab shows it), and the pick goes on to the next element.
`Backspace` clears the binding of the picked element, `F4` or `Esc` leave the mode. While
the mode is open the keyboard and controller pads do not sound and the commands of the
screen do not act; the help line names the mode keys.

#### Scenario: A knob onto the volume
- **WHEN** the owner presses `F4` on the pads screen, walks to "volume" in the panel and turns knob k5
- **THEN** the label says `k5`, the pick moves to "pitch", and after `F4` knob k5 turns the volume of the selected pad

#### Scenario: Controller pads onto the grid
- **WHEN** the mode is open on pad 1 of the grid and the owner hits controller pads 1–4 in turn
- **THEN** pads 1–4 of the grid are labelled with them, and after leaving each controller pad plays its pad

#### Scenario: A command
- **WHEN** the mode is open on the chop screen, the pick is on "cut" in the help line and the owner presses controller button b6
- **THEN** after leaving the mode b6 cuts at the cursor on the chop screen

#### Scenario: Clear
- **WHEN** the pick is on an element bound to k5 and the owner presses `Backspace`
- **THEN** the label says `---` (or the profile's control again) and k5 no longer acts on it

#### Scenario: Silent while learning
- **WHEN** the mode is open and the owner hits a keyboard pad key
- **THEN** no pad sounds and the pick stays where it was

### Requirement: The walk goes by zones
The system SHALL walk the elements of the learn mode by zones in this order: the grid by
rows, the bank switch, the panel top to bottom, the commands of the help line left to
right. `J` `L` (`←` `→`) move inside a zone, `I` `K` (`↑` `↓`) jump to the previous or the
next zone; a binding moves the pick to the next element, past the end of a zone into the
next one.

#### Scenario: Through the grid
- **WHEN** the pick is on pad 4 and the owner presses `L`
- **THEN** the pick is on pad Q, the first of the second row

#### Scenario: To the panel
- **WHEN** the pick is on a pad and the owner presses `K` twice
- **THEN** the pick is on the first parameter of the panel

### Requirement: A binding goes to the layer in sight
The system SHALL save a binding made in the learn mode into the layer the screen is in:
the edit layer of the screen while its edit is open (28), else the screen's group.

#### Scenario: In the edit
- **WHEN** the pad edit is open on ENV, the owner binds k1 to "attack" in the learn mode and leaves it
- **THEN** k1 turns the attack while the edit is open and what it turned before with the edit closed

#### Scenario: One screen at a time
- **WHEN** the owner binds button b1 to "next bank" on the pads screen
- **THEN** b1 turns the bank on the pads screen, and on the chop screen it does what it did before

## ADDED Requirements

### Requirement: Edit mode
The system SHALL open an edit mode for the selected object with `Enter` and close it with
`Esc`, a second `Enter` or by switching the screen. In the edit mode the page knobs turn the object's
parameters; pads keep playing, and a pad hit still sounds and selects its pad. The edit
mode and the assign mode never stand together. `Esc` closes the help (F1) before the edit
mode.

#### Scenario: Enter opens, Esc closes
- **WHEN** a pad is selected on the pads screen and the owner presses `Enter` and then `Esc`
- **THEN** while the mode is open the screen names the page and what every knob turns, and after `Esc` the knobs do what they did before

#### Scenario: The music plays on
- **WHEN** a pad sounds and the owner enters the edit mode and turns the decay knob
- **THEN** the sound keeps playing and the decay changes on it without a click

#### Scenario: Esc closes the help first
- **WHEN** the edit mode is open, the owner opens the help with F1 and presses `Esc`
- **THEN** the help closes and the edit mode stays open

### Requirement: Parameter pages
The system SHALL group the parameters of the edited object into pages of at most 3; knob k
turns the k-th parameter of the current page, on every screen. `Tab` and `I` `K` (`↑` `↓`)
walk all parameters of the object; the page follows the picked parameter. The pads screen has TONE
(volume, pitch) and ENV (attack, decay, release); the chop screen has BOUNDS (slice start,
slice end).

#### Scenario: The envelope page fills three knobs
- **WHEN** the picked parameter of a pad is attack, decay or release
- **THEN** the page is ENV and the knobs turn attack, decay and release, in this order

#### Scenario: Tab moves to the tone page
- **WHEN** the owner presses `Tab` until volume is picked
- **THEN** the page is TONE and the knobs turn volume and pitch

#### Scenario: Up down walks the parameters
- **WHEN** the edit mode is open and the owner presses `K` and then `I`
- **THEN** the next parameter is picked and then the previous one, and the page follows
  each pick

### Requirement: Key roles
The system SHALL split the work between the hands and give every key one meaning per
layer: the left fingers play the pads, the left thumb holds `Ctrl` and `Alt` for the
right hand's keys; the right hand moves and changes with the direction letters `J` `L`
`I` `K` (the arrows are the same keys) and gives the commands. In the edit mode `I` `K`
pick the parameter and `J` `L` change the picked value on every screen; outside it they
walk what the screen shows (the chop cursor and zoom, the banks on the pads screen). A
tap is the fine step, holding repeats the coarse step. A command letter means the same on
every screen it is bound on; no command sits on the left half of the keyboard. `−` `=`
and the arrows do not pick a pad.

#### Scenario: The same keys on a pad and on a slice
- **WHEN** the owner edits a pad on the pads screen and a slice on the chop screen
- **THEN** `I` `K` pick the parameter and `J` `L` change it on both screens, and the
  arrows do the same as the letters

#### Scenario: A tap is fine, holding is coarse
- **WHEN** the edit mode of a pad is open on the attack and the owner taps `L`, then
  holds it
- **THEN** the tap raises the attack by 1 ms and every repeat of the held key by 5 ms

#### Scenario: Arrows change the picked value
- **WHEN** the edit mode is open and the owner presses `→`
- **THEN** the picked value moves a fine step and the selection stays

#### Scenario: Enter closes too
- **WHEN** the edit mode of a pad is open and the owner presses `Enter`
- **THEN** the mode closes, as with `Esc`

#### Scenario: Minus and equal are free
- **WHEN** no edit mode is open and the owner presses `−` on the pads screen
- **THEN** nothing changes

#### Scenario: No command on the left half
- **WHEN** the default keymap is loaded
- **THEN** no command is bound to `1`–`5`, `Q`–`T`, `A`–`G`, `Z`–`B` without `Ctrl` or
  `Alt`, and every command letter but the direction letters `J` `L` `I` `K` and the marker
  key `M` means the same on every screen it is bound on

### Requirement: The screen shows the layer
The system SHALL show at all times which layer is active, which object and which
parameter are picked, and, in the edit mode, which parameter every knob turns now:

- the edit mode changes the frame colour of the panel it edits and names the mode and
  the page in the frame title (`EDIT · ENV`);
- the picked parameter is highlighted in the panel; on the chop screen the bound that the
  keys move now is drawn in its own colour, the lines stay thin: the bound of a held
  modifier (`Ctrl` the left one, `Alt` the right one), else the picked bound in the edit
  mode, else the bound that just moved;
- in the edit mode every parameter of the page carries its knob number in the panel;
- a knob turn or a value key lights the changed parameter for about a second, in and out
  of the edit mode;
- the help line of an edit mode says that a pad hit listens.

#### Scenario: The frame tells the mode
- **WHEN** the owner presses `Enter` on a pad and then `Esc`
- **THEN** the pad panel frame changes colour and its title says `EDIT · TONE`, and after
  `Esc` the frame is as before

#### Scenario: The knob numbers
- **WHEN** the pad edit mode is open on the ENV page
- **THEN** the panel shows the knob number 1 next to attack, 2 next to decay, 3 next to
  release, and the help line names the ENV page

#### Scenario: A knob lights what it turns
- **WHEN** the owner turns the knob bound to the pad volume outside the edit mode
- **THEN** the volume line of the pad panel lights for about a second

#### Scenario: The picked bound is visible
- **WHEN** the slice edit is open and the slice end is the picked parameter
- **THEN** the end line of the selected slice is drawn in the bound colour, its start
  line in the slice colour, both thin

#### Scenario: A held modifier lights its bound
- **WHEN** the chop screen is open and the owner holds `Ctrl`, lets it go and holds `Alt`
- **THEN** the start line of the selected slice takes the bound colour while `Ctrl` is
  held, then the end line while `Alt` is held; no line gets thicker; when the cursor
  stands on that line, the cursor takes the bound colour too, so the lit bound never
  hides under it

## MODIFIED Requirements

### Requirement: Parameter pages
The system SHALL group the parameters of the edited object into pages of at most 3; knob k
turns the k-th parameter of the current page, on every screen. `Tab` and `I` `K` (`↑` `↓`)
walk all parameters of the object; the page follows the picked parameter. The pads screen has TONE
(volume, pitch), ENV (attack, decay, release), SOURCE (source, start, end) and FILTER
(filter, cutoff, reso); the chop screen has BOUNDS (slice start, slice end).

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

#### Scenario: The filter page
- **WHEN** the owner presses `Tab` past the end of the SOURCE page
- **THEN** the page is FILTER and the knobs turn the filter type, the cutoff and the
  resonance, in this order

### Requirement: The screen shows the layer
The system SHALL show at all times which layer is active, which object and which
parameter are picked, and, in the edit mode, which parameter every knob turns now:

- the edit mode changes the frame colour of the panel it edits and names the mode and
  every page of the object in the frame title, the current page lit
  (`EDIT · TONE ENV SOURCE FILTER`), so the owner sees where `I` `K` and `Tab` lead;
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
- **THEN** the pad panel frame changes colour and its title says `EDIT · TONE ENV SOURCE FILTER`
  with TONE lit, and after `Esc` the frame is as before

#### Scenario: The title shows where Tab leads
- **WHEN** the pad edit is open on TONE and the owner presses `Tab` until FILTER
- **THEN** the lit page in the title moves from TONE to ENV to SOURCE to FILTER, the other pages stay
  named in it

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

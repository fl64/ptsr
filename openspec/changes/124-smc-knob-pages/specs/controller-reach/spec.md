## ADDED Requirements

### Requirement: SMC-PAD Chop zoom uses smaller relative steps

On Chop and Chop edit, bank 1 knob 2 SHALL change zoom in quarter steps on the log scale.
The turn SHALL keep the cursor anchor and current limits.
The knob SHALL apply movement immediately and retain acceleration.
Keyboard and other device zoom routes SHALL keep their step sizes.
Assign and modal input rules SHALL keep their priority.

#### Scenario: Slow Chop zoom uses smaller input steps
- **WHEN** the owner sends four isolated movement steps on bank 1 knob 2
- **THEN** each step changes the zoom by a quarter of a keyboard step on the log scale
- **AND** the four steps give one keyboard step within frame rounding
- **AND** reverse turns act at once and fast turns retain acceleration

#### Scenario: SMC-PAD Chop zoom preserves Assign and question priority
- **WHEN** the owner turns bank 1 knob 2 during Assign or a question
- **THEN** the current view stays in place and the mode keeps control

### Requirement: Chop knobs provide separate slice and region pairs

On Chop and Chop edit, bank 1 knobs 3 and 4 SHALL change the slice start and end.
Knobs 5 and 6 SHALL change the Chop region start and end.
The pairs SHALL keep the current limits, links and Undo rules.
Assign and visible questions SHALL block bound turns.

#### Scenario: Chop knobs move two separate bound pairs
- **WHEN** the owner turns bank 1 knobs 3–6 on Chop or Chop edit
- **THEN** each turn moves only the named edge
- **AND** Undo restores the change with its affected pad links

#### Scenario: Chop bounds wait during Assign and questions
- **WHEN** the owner turns a bound knob during Assign or a visible question
- **THEN** the bounds, plan and question stay in place

## MODIFIED Requirements

### Requirement: Chop threshold has matching keyboard steps

On Chop and Chop edit, Ctrl+Alt+Up SHALL increase the hit threshold by 1 dB.
Ctrl+Alt+Down SHALL decrease it by 1 dB.
The keys SHALL keep their existing threshold actions.
An isolated bank 1 knob 7 step SHALL use the same threshold interval.
Fast turns SHALL keep their current acceleration.
The threshold SHALL stay within -40 to 0 dB.

#### Scenario: Keyboard and encoder set the same Chop threshold
- **WHEN** keyboard and raw MIDI runs step the threshold in both directions at least 100 ms apart
- **THEN** each run reaches the same threshold, cuts, linked pads and Undo state
- **AND** an attempt to pass a bound creates no Undo entry

#### Scenario: Fast encoder turns keep acceleration
- **WHEN** bank 1 knob 7 sends two movement steps within 40 ms on Chop
- **THEN** the existing MIDI speed rule accelerates the second step
- **AND** two separate keyboard presses still change the threshold by exactly 2 dB

#### Scenario: Threshold keys respect owner bindings and input owners
- **WHEN** an owner replaces or removes a threshold key, or an input overlay owns input
- **THEN** the owner entry or active mode keeps priority

### Requirement: Context page selection preserves the screen controls

On Chop and Chop edit, first-bank knob 6 SHALL keep its region-end role.
On other screens, knob 6 SHALL select a context page only while the panel shows.
The program SHALL show the selected page and its labels before another cell acts.
More and one context cell SHALL reach a first context page action.
On Chop and Chop edit, the Next cell SHALL move to later context pages.

#### Scenario: Encoder selects a later context page
- **WHEN** the panel shows a context page outside Chop and Chop edit and the owner turns knob 6
- **THEN** the later page labels show before another cell acts
- **AND** the screen value and bank stay in place

#### Scenario: A Chop bound turn keeps the current panel page
- **WHEN** the owner turns knob 6 with the Chop context page visible
- **THEN** the turn changes the region end and keeps the panel page
- **AND** the Next cell moves to a later context page

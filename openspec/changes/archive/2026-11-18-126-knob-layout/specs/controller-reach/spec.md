## ADDED Requirements

### Requirement: Bank 2 has only the master role

The shipped bank 2 knobs 1–7 SHALL have no role.
Bank 2 knob 8 SHALL change master volume.
An owner binding on bank 2 SHALL keep its action.

#### Scenario: Bank 2 turns change no pad value
- **WHEN** the owner selects a pad and turns bank 2 knobs 1–7 on each screen
- **THEN** no pad value, page, bank or selection changes
- **AND** bank 2 knob 8 changes master volume

### Requirement: Pad edit knobs follow the screen rows

On Pads outside pad edit, bank 1 knobs 1–7 SHALL have no role.
The pads select a pad. The A1 overview keeps its knob 1 step route.
A pad edit page SHALL have at most four values.
In pad edit, bank 1 knobs 7, 5, 3 and 1 SHALL change the first to fourth value.
Knobs 2, 4 and 6 and an absent value SHALL have no role in pad edit.
LEFT and RIGHT SHALL select the page.
A pad value turn SHALL use the target, scope and limits of the keyboard value keys.
An FX turn SHALL change the current FX slot.

#### Scenario: Pad knobs act only in pad edit
- **WHEN** the owner turns bank 1 knobs 1–7 on Pads without pad edit
- **THEN** no pad value changes
- **AND** the selected pad stays the same

#### Scenario: Pad edit knobs follow the screen rows
- **WHEN** the owner opens pad edit, selects each page with LEFT or RIGHT and turns knobs 7, 5, 3 and 1
- **THEN** the knobs change the page values in screen order from top to bottom
- **AND** knobs 2, 4 and 6 and a knob without a value change nothing

### Requirement: Chop threshold steps match the keyboard

On Chop and Chop edit, Ctrl+Alt+Up SHALL increase the hit threshold by 1 dB.
Ctrl+Alt+Down SHALL decrease it by 1 dB.
The keys SHALL keep their existing threshold actions.
Each bank 1 knob 7 step SHALL use the same 1 dB interval at any turn speed.
The threshold SHALL stay within -40 to 0 dB.

#### Scenario: Keyboard and encoder set the same Chop threshold
- **WHEN** keyboard and raw MIDI runs step the threshold in both directions at least 100 ms apart
- **THEN** each run reaches the same threshold, cuts, linked pads and Undo state
- **AND** an attempt to pass a bound creates no Undo entry

#### Scenario: Fast encoder turns use single threshold steps
- **WHEN** bank 1 knob 7 sends two movement steps within 40 ms on Chop
- **THEN** the threshold changes by exactly 2 dB
- **AND** two separate keyboard presses give the same threshold

#### Scenario: Threshold keys respect owner bindings and input owners
- **WHEN** an owner replaces or removes a threshold key, or an input overlay owns input
- **THEN** the owner entry or active mode keeps priority

## MODIFIED Requirements

### Requirement: Pads have a silent keyboard selection route

On Pads and Pads edit, `Alt+Left` and `Alt+Right` SHALL select the adjacent pad in the same row. Selection at a row edge SHALL stay in place. These keys SHALL not hit a pad, change its value, stop a held Gate pad or end the splash. A key repeat SHALL not select another pad. Other screens SHALL keep their current Alt-arrow routes. Owner keymap entries and an active text line, question, picker, Help or learning mode SHALL keep priority.

#### Scenario: Keyboard selects a neighboring pad without sound
- **WHEN** the owner presses `Alt+Left` or `Alt+Right` on Pads or Pads edit
- **THEN** the adjacent pad in the same row becomes the selected pad
- **AND** no pad plays, a held Gate keeps its release target and a repeat does not move again
- **AND** selection stops at the row edge without changing a value or ending the splash

#### Scenario: Silent selection respects owner bindings and overlays
- **WHEN** an owner keymap replaces an Alt-arrow key or sets it to `none`, or an input overlay owns the key
- **THEN** the owner entry or overlay acts first and the pad selection stays unchanged
- **AND** Chop keeps its existing Alt-arrow slice-bound route

### Requirement: Pattern transport supplies the shipped tempo knob

Pattern transport SHALL provide tempo, Bars, Quantize and Strength on its current page.
Each tempo step SHALL change 1 BPM, or 0.1 BPM while the owner holds fine mode.
The shipped SMC-PAD map SHALL have no tempo knob or step cell outside this page.
The keyboard and learned tempo actions SHALL retain their routes.

#### Scenario: Tempo turns act only on the Pattern transport page
- **WHEN** the owner turns the tempo position with and without Pattern controls open
- **THEN** the turn changes tempo only while the Pattern controls are open
- **AND** each step uses the same value as the 1 BPM keyboard step

### Requirement: SMC-PAD knob pairs share a physical row

The device layout SHALL mark its symmetric knob pairs.
Each pair SHALL use adjacent knobs in one physical row.
The lower, earlier or first role SHALL be on the left.
Pad edit values SHALL follow the screen column, not pairs.

#### Scenario: Layout rejects a split or reversed knob pair
- **WHEN** a shipped pair moves apart or reverses its roles
- **THEN** the layout check reports the pair and its positions

### Requirement: SMC-PAD Chop zoom uses smaller relative steps

On Chop and Chop edit, bank 1 knob 2 SHALL change zoom in quarter steps on the log scale.
The turn SHALL keep the cursor anchor and current limits.
The knob SHALL apply each step immediately at any turn speed.
Keyboard and other device zoom routes SHALL keep their step sizes.
Assign and modal input rules SHALL keep their priority.

#### Scenario: Slow Chop zoom uses smaller input steps
- **WHEN** the owner sends four isolated movement steps on bank 1 knob 2
- **THEN** each step changes the zoom by a quarter of a keyboard step on the log scale
- **AND** the four steps give one keyboard step within frame rounding
- **AND** reverse turns act at once and fast turns use the same step

#### Scenario: SMC-PAD Chop zoom preserves Assign and question priority
- **WHEN** the owner turns bank 1 knob 2 during Assign or a question
- **THEN** the current view stays in place and the mode keeps control

## REMOVED Requirements

### Requirement: Bank 2 follows the selected pad on each screen

**Reason**: Change 126 removes the bank 2 pad roles. A pad value had two knobs.
**Migration**: Use pad edit and bank 1 knobs 7, 5 and 3.

### Requirement: Bank 1 uses the current Pads and FX page positions

**Reason**: Change 126 removes the pad value roles outside pad edit and puts the pad edit values in screen order.
**Migration**: `Pad edit knobs follow the screen rows` gives the new roles.

### Requirement: Chop threshold has matching keyboard steps

**Reason**: Change 126 removes knob acceleration from the threshold steps.
**Migration**: `Chop threshold steps match the keyboard` keeps the keyboard and knob steps.

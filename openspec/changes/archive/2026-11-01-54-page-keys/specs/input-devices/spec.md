## MODIFIED Requirements

### Requirement: Edit layer on device buttons
The system SHALL let device buttons do what the edit keys do, so the whole edit loop runs
from a controller without the keyboard: a screen button opens the edit (`pad.edit`,
`chop.edit`, as `Enter`), and the `[edit]` layer of the profile, active while an edit mode
is open, binds closing it (`ui.back`), the page turn (`edit.page_prev`, `edit.page_next`,
as `Shift+Tab` `Tab`; the page knob `edit.page`) and the parameter pick (`edit.param_prev`, `edit.param_next`, as
`↑` `↓`). The layer is the same on every screen and stands over the screen roles and the
learned controls while the mode is open. The X-Session Pro profile puts the toggle on b10,
the page turn on b3 and b4, the page on k1–k3, and no page knob: the owner binds one in
F12 if wanted.

#### Scenario: Edit a pad from the controller only
- **WHEN** a pad is selected and the owner presses b10, then b4, turns knob 1 and presses b10
- **THEN** the edit mode opens, the ENV page is shown, the attack follows the knob, and
  the edit mode closes

#### Scenario: Pages from the controller
- **WHEN** the pad edit is open on TONE and the owner presses b4 twice and then b3
- **THEN** the page is ENV, then FILTER, then ENV again, and knobs 1–3 turn the
  parameters of the page shown

#### Scenario: The buttons get their roles back
- **WHEN** the edit mode is closed on the chop screen and the owner presses b3
- **THEN** the previous slice is selected, as before the edit

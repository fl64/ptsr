# Design

## Context and evidence

Change 91 maps Master notes 52 through 67 to `c1` through `c16` in screen row order. Those controls have no starting commands. The profile has guarded relative encoders and keeps the mirrored Private port silent. Physical presses of all second-layer pads and SHIFT remain unverified.

The current transport `Stop` resets the clock immediately (`openspec/specs/transport/spec.md`, "Transport play and stop"). The planned SMC-PAD STOP role says that its first tap stops and its second tap returns to the start. The existing toggle and explicit stop cannot show two distinct results. This change adds a pause and continue path for SMC-PAD. Space keeps its current toggle and reset behavior.

The cross-cutting plan assigns full Redo and Panic behavior to change 93a. This change reserves their two fixed panel cells and shows them as unavailable. It does not pretend that a disabled cell acts.

## Decisions

### Panel entry and ownership

Only the Master profile's panel-bound `c1` through `c16` controls enter the panel. A first panel-bound press while the panel is hidden shows it and runs no cell. Its release does nothing. Later presses run one visible cell each. An explicit owner action on a cell acts outside a question and bypasses panel entry. A first-layer Master pad press hides the panel and follows its existing pad route on that same press. The panel does not take ownership of its held Gate release. PAD BANK itself has no host event. A keyboard screen change updates the panel's screen context without changing its device owner.

The panel belongs to the Master connection that showed it. A second device's pads keep their own routes. A disconnect clears that panel and its badge. A reconnect does not inherit its visible state. Invalid shared MIDI state refuses new panel presses and preserves accepted Gate releases.

### Cells and pages

Cells use the screen's physical 4 by 4 order: `c1` through `c4` top, `c13` through `c16` bottom. The bottom row is fixed: Back, Undo, Redo and Panic. Redo and Panic are dim until change 93a. A dim cell runs no action.

The main page uses `c1` through `c4` for Pads, Files, Chop and Pattern. It uses `c5` through `c8` for FX, Settings, Help and More. It uses `c9` through `c12` for four current-screen actions. More opens the first context page. Each context page has up to ten actions, Next in `c11` and Main in `c12`. Next cycles through context pages. Main returns to the screen page. Pages always show their title and a page count. Switching screens or leaving the panel selects the main page. LEFT and RIGHT keep their previous and next item roles while a page shows.

The main page uses these short labels: `pads`, `files`, `chop`, `ptrn`, `fx`, `sets`, `help`, `more`. F5's `screen.pattern` action opens the existing A1 overview on Pads. It is not a new screen. A missing context action leaves its cell dim.

| Current view | Main cells 9 through 12 | First context page, cells 1 through 10 |
| --- | --- | --- |
| Pads | `pad.edit`, `pad.copy`, `pad.take`, `pad.mark_toggle` | `pad.mode_next`, `pad.mark_bank`, `pad.clear`, `pattern.controls`, `pattern.record`, `pattern.erase`, `pad.pitch_gate_toggle`, `ui.pads_toggle`, `project.pick`, `session.save` |
| Pad edit | `edit.page_prev`, `edit.page_next`, `edit.param_prev`, `edit.param_next` | `edit.value_dec`, `edit.value_inc`, `pad.mode_next`, `pad.clear`, `pad.pitch_gate_toggle`, `session.save` |
| A1 overview | `pattern.step_edit`, `pattern.view_prev`, `pattern.view_next`, `pattern.controls` | `pattern.record`, `pattern.erase`, `pattern.undo`, `pattern.clear`, `pattern.overview_empty_toggle`, `pattern.param_prev`, `pattern.param_next`, `pattern.value_dec`, `pattern.value_inc`, `session.save` |
| Files | `browser.hold`, `browser.load`, `browser.parent`, `browser.favorite_toggle` | `browser.enter`, `browser.preview_toggle`, `screen.chop`, `project.pick`, `session.save` |
| Chop | `chop.cut`, `chop.parts`, `chop.hits`, `chop.assign` | `chop.loop_toggle`, `chop.play_mode`, `chop.snap_toggle`, `chop.as_pad_toggle`, `chop.normalize`, `chop.commit`, `chop.region_start`, `chop.region_end`, `chop.region_clear`, `chop.edit` |
| Chop edit | `edit.page_prev`, `edit.page_next`, `edit.param_prev`, `edit.param_next` | `edit.value_dec`, `edit.value_inc`, `chop.normalize`, `session.save` |
| FX | `edit.page_prev`, `edit.page_next`, `edit.param_prev`, `edit.param_next` | `edit.value_dec`, `edit.value_inc`, `session.save` |
| Settings | `settings.tab_next`, `settings.learn`, `settings.clear`, `session.save` | `settings.row_prev`, `settings.row_next`, `settings.value_prev`, `settings.value_next`, `project.pick` |

The configuration can add further context pages. The shipped map needs only one context page per view. An isolated configuration fixture tests a third page and the `Next` cycle. Cells use the action labels of their current view, not raw MIDI numbers. Page changes themselves use no logical sampler action.

The shipped `config/panel.toml` gives labels and logical action names per screen and mode. The loader checks cell ranges, duplicate cells, labels of at most six terminal cells, known actions and fixed bottom cells. An invalid entry disables that cell and names the file and cell in a notice. Other cells and pad play keep working. A cell dispatches through the normal action path. It does not call an audio or application method directly.

The existing profile initialization rule leaves previously connected devices' saved bindings untouched. For this change, a missing SMC-PAD binding for a new panel cell or PLAY/STOP role uses the shipped profile default at runtime. An explicit owner binding, including `none`, wins. The program does not rewrite saved bindings. This exception is limited to the new SMC-PAD controls and is specified in the `input-devices` delta.

The saved MIDI map already stores action names as strings. This change accepts `none` as an explicit no-action name only for the new SMC-PAD controls. F12 clear removes that entry, so the shipped fallback returns. Other devices keep their current seeding and clearing rules.

### Questions and input priority

A visible question takes priority over the ordinary panel map and owner cell bindings. Physical P7 (`c7`) cancels and P8 (`c8`) confirms it on the first press, even when the panel was hidden. The first-hit reveal rule does not consume either answer. Other second-layer cells do not answer or reveal a panel during a question. Destructive cells only open the normal visible question. They never answer it with the same press. A shipped PLAY or STOP context transport role can act under a question. An owner `ui.confirm` binding on PLAY or STOP cannot answer it. A question for replacing a sample or applying a Chop layout keeps its existing target checks.

Text input, pickers, MIDI learn and screen learn keep their existing priority. A panel press cannot submit typed text or become a pad hit. During a question, only P7 and P8 answer it. Outside a question, Back closes the highest-priority open mode before it closes the panel. Help and Messages retain their current navigation and content priority.

### Button roles and transport

PLAY uses `play.context`: on Pads, Pattern, FX and Settings it continues a paused transport or starts from bar 1. On Files it starts the selected preview only if none plays. On Chop it starts the selection preview only if none plays. Holding PLAY for the existing 300 ms threshold toggles the metronome. A PLAY tap does not stop anything already playing.

STOP uses `stop.context` on press with no starting hold. On Pads, Pattern, FX and Settings its first tap pauses the transport at its accepted position. Its next tap while paused resets position to bar 1. PLAY after the first tap continues that position. STOP also stops a Files or Chop preview without changing transport. A second STOP tap without an active preview resets transport only if it is paused. These actions use accepted audio state, not a guessed UI clock. Change 93a can add an optional STOP hold setting. RECORD keeps its measured microphone hold. Its pattern-record tap acts only where the existing recording action is available. LEFT and RIGHT keep the current selection roles and key-repeat behavior.

The keyboard keeps Space and preview keys. `Alt+T` invokes the same context PLAY action and `Alt+Y` invokes the same context STOP action. Neither key is in the pad block. Owner key overrides keep their priority. Change 93 checks all action paths from both inputs.

### Drawing and sound

The panel replaces the pad grid at 80 by 26 and 120 by 40. Its cells use the same positions as the playing pads. The title shows `PANEL · SMC`. The status shows the active page after a second-layer hit. The first reveal changes no audio or transport state. A question stays above the panel. The panel's cell labels and current button roles appear in F1. The existing image-overlay rules hide intersecting kitty images before text draws and restore them after the panel closes.

The offline render checks A1 pause, continuation from its held tick, second-STOP reset and the absence of extra hits at the boundaries. It also checks finite samples, clipping, silence and clicks. The callback benchmark uses its existing 32-voice workload after the transport change.

## Risks and edge cases

| Situation | Expected result | Status |
| --- | --- | --- |
| Second-layer press and its release arrive before redraw | Reveal once and run no cell. Release has no effect. | Support now. Raw MIDI test required. |
| Two second-layer presses arrive before redraw | First reveals. Second runs one cell in input order. | Support now. Raw MIDI test required. |
| Question opens from a panel cell and P8 arrives in the same batch | P8 confirms only the visible question. The first cell does not confirm it. | Support now. Raw MIDI test required. |
| Hidden panel receives P7 or P8 while a question shows | Answer the question on that first press | Support now. Raw MIDI test required. |
| First-layer Gate hit follows a panel command | Hide the panel, sound once and retain its original release | Preserve existing release contract. Raw MIDI test required. |
| X-Session input arrives while the SMC panel shows | Its own pad and action routes stay independent | Preserve existing behavior. Two-device test required. |
| SMC-PAD disconnects with panel and held Gate | Clear panel and release only its Gate | Support now. Disconnect test required. |
| STOP arrives while a question shows | Change transport or preview, never answer the question | Support now. Keyboard and MIDI test required. |
| Audio accepts pause after a delayed UI frame | Show only accepted position and resume there | Support now. Offline render required. |
| SHIFT with a physical pad changes the device preset | No host SHIFT shortcut can run or change a panel cell | No measured host message. Hardware check remains open. |

## Open questions

- Does the first physical STOP need to pause at the current position? Default: yes. The owner was asked because the existing transport specification resets on its first explicit stop. Keep Space unchanged.
- Does the optional STOP hold need a saved setting in change 92? Default: no. Keep it off and defer its setting to the grouped controller settings in change 93a. The main PAD BANK path works without it.
- Can an explicit owner `pad.1` assignment on a second-bank cell sound? Default: yes, outside a question. It replaces that cell's panel entry until the owner clears the binding.
- Can an explicit owner `ui.confirm` assignment on PLAY answer a question? Default: no. Physical P7/P8 answer the question, and PLAY/STOP cannot confirm it.

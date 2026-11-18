# Design

## Context and evidence

Change 92 supplies the panel and its first-hit rule. `config/panel.toml` gives one context page per view. In `crates/app/src/dispatch.rs`, `PanelCell` accepts only Back while a picker or Help is open. The project-list step target exists, but the shipped SMC-PAD profile does not bind it in those modes. `crates/input/src/lib.rs` excludes `app.quit` from `ACTIONS` so that MIDI learn cannot offer it. Keep that MIDI learn rule.

`openspec/specs/screen-layout/spec.md` says that sound, not a silent input, ends the splash. The planned suggestion to close it on any controller input conflicts with this accepted requirement. The splash does not block commands. Change 93 keeps the current splash rule and updates the plan.

## Decisions

### Reach inventory

The reach inventory starts with effective keyboard actions on each screen and in each mode. It also includes panel cells, profile buttons and knobs that have no direct keyboard binding. Tests use the shipped `config/keymap.toml`, `config/devices/smc-pad.toml` and `config/panel.toml`, then send recorded keyboard events and raw MIDI through production dispatch. An action name in a TOML file alone is not evidence that a modal handler accepts it. The test observes the result and the mode badge.

The current `ACTIONS` catalogue has 209 entries, 171 distinct names and 50 knob entries. The panel map has eight views with a main page and one context page each. First-bank encoders 6, 7 and 8 have no shipped role. These counts describe configuration and metadata, not successful routes.

| Screen or mode | Confirmed gap in the shipped route | Status |
| --- | --- | --- |
| Project picker over any screen | Encoder 1 does not consistently select the project. Panel P8 does not confirm it. | Open in 93. |
| Paths folder picker | Encoder steps stop before folder selection. LEFT and RIGHT use the Settings row route, not parent and child navigation. | Open in 93. |
| Settings values | Encoder 2 has no shipped value role. | Open in 93. |
| Help overlay | Panel cells other than Back cannot change the Help tab or selection. | Open in 93. |
| Every panel view | Only one context page exists. Encoder 6 cannot select later pages. | Open in 93. |
| Session exit | Ctrl+Q exits directly. No SMC-PAD Exit cell or confirmation question exists. | Open in 93. |
| Global commands | Output recording and screen learning have no shipped SMC-PAD cell. | Open in 93. |
| Pad exchange on other screens | Ctrl+X can take the selected pad on Settings, but the shipped panel has Take only on Pads. | Add Take to common page 2 in 93. |
| Chop | Hits Back and both cut-deletion commands have no shipped SMC-PAD cell. | Open in 93. |
| A1 controls over any screen | The shipped `[screens.pattern]` encoder roles do not load. Encoder 2 does not change Bars, while the keyboard value key does. | The owner approved a distinct MIDI context for the open A1 controls on 2026-11-17. Fix in 93. |

The reach test must decide whether another keyboard action has an equivalent knob, button or context route before it closes this list. A static comparison of action names cannot decide that question.

The page 2 `trnspt` cell runs `transport.toggle` on Files and Chop. Space runs the preview action on these screens. The shipped keyboard map has no other effective key for `transport.toggle` there. The owner approved `Alt+U` on 2026-11-17 as a shared transport key. Keep Space previews and the existing owner keymap priority. Test the result from the same screen through keyboard and raw SMC-PAD input, in both supported keyboard layouts. A keymap entry in a different screen does not count as a route.

The static inventory treated SMC-PAD LEFT and RIGHT on Pads as covered by a keyboard pad hit. The button selects a neighboring pad without sound, but a keyboard pad hit also plays it. Bind `Alt+Left` and `Alt+Right` to `pad.select_left` and `pad.select_right` only on Pads. The screen binding also works in Pads edit, while plain arrows keep their value route. Chop keeps its Alt-arrow slice-bound keys. Test both inputs from the same selected pad, including row edges, a held Gate, key repeat, owner overrides and active overlays. A silent selection does not end the sound-first splash.

An isolated SMC-PAD encoder 3 detent changes the Chop hit threshold by 1 dB. The existing MIDI speed rule accelerates detents within 40 ms. `H` and `Shift+H` select fixed thresholds. The owner approved exact keyboard steps on 2026-11-17. Add `chop.threshold_up` and `chop.threshold_down` to the Chop and Chop edit action groups. Bind `Ctrl+Alt+Up` to +1 dB and `Ctrl+Alt+Down` to -1 dB on Chop and Chop edit. Send both keys and encoder 3 through the current threshold setter. Keep its -40 to 0 dB bounds, recut, linked-pad and Undo behavior. At a bound, a step does not add an Undo entry. Test isolated encoder detents at least 100 ms apart against key presses through production dispatch. Also test both Chop modes, both bounds, Undo, owner overrides and modal priority. The new chords also need a legacy-terminal path under the accepted keyboard contract.
The action list uses `th+1` and `th-1` as short labels for these keys.

The owner approved a distinct MIDI context for open A1 controls on 2026-11-17. Load `[screens.pattern]` as an overlay context, not as a replacement for the underlying keyboard screen. Encoders 2, 3 and 4 change Bars, Quantize and Strength only while the controls are open. Encoder 1 keeps its shipped bank role. A profile's pattern roles have their own learned group, so owner pattern assignments win over the shipped roles. Global controls, panel cells and pad releases keep their current routes. When the controls close, the underlying screen roles return. A question, picker, text line or learning mode keeps priority. Seed the group for a new SMC-PAD. For an already seeded device, add the shipped group only when it is absent. An existing group, even an empty one, can contain the owner's removals and stays unchanged. Show the current encoder roles in the Controller tab. Test the raw profile on every underlying screen, both seed states, owner overrides, close, modals, a second device and a held Gate release.

`ActionInfo` does not yet mark text entry. Add that metadata for `pad.rename` and `browser.filter`. A typed new-project name and keymap edits are document operations, not `ActionInfo` entries. `app.quit` stays outside `ACTIONS` and MIDI learn. The SMC-PAD panel offers Exit as an explicit context cell. No other action gets a text exception. The inventory separates a parameter turn from a button action and names equivalent routes when their action IDs differ.

The story table counts physical presses from the visible main panel and from a hidden panel. A visible context action takes More and its cell. Knob turns do not add presses. The first entry adds PAD BANK and a reveal hit with the shortcut off. With the shortcut on, count the shorter path when the panel opens before the first pad hit. Record the first-hit fallback too. Record a target pad hit or a later modal answer separately. A story that only describes visual state has zero command presses. An X-Session setup story names that device and is not presented as an SMC-PAD action.

### Context pages and modal controls

The current main page and fixed bottom cells remain. More opens the context page. If the inventory needs more context pages, the unused first-bank encoder 6 selects a context page while the SMC panel is visible. The selected page and its cell labels update before the next hit. This keeps a context action within More plus one cell hit from the visible main panel. With the panel hidden, encoder 6 retains its existing unassigned result. The keyboard keeps its direct action paths. This page-selection gesture is navigation within the SMC panel, like PAD BANK selection, and does not change a sampler parameter.

The relative-step event currently omits its source device. Add the source port and control to that event before it drives modal selection or panel page selection. Only the SMC-PAD Master port that owns the visible panel can change its context page. The profile gives encoder 6 a panel-page target, not an edit-page target. A second device keeps its own route. Encoder 1 selects the active project or folder in a picker before it changes the screen behind that picker. Encoder 2 changes a Settings value only without a picker.

Modal handlers take priority over the underlying panel. In a project picker, encoder 1 moves the project selection, P8 confirms and P7 or Back cancels. In the Paths folder picker, encoder 1 moves the row. LEFT and RIGHT move to parent and child folders. P8 selects the current folder, and P7 or Back cancels. A visible question still takes P7 and P8 before either picker. The first-hit reveal rule does not consume a modal answer. A failed project save keeps the picker and selection. A held first-bank Gate receives its original release.

In Settings, encoder 2 changes the selected value. It uses the same action and bounds as the keyboard value keys. When a folder picker opens, the encoder moves only the picker and does not change a setting. The current screen's other knobs keep their roles. The input strip names encoder 2's Settings role.

The main Help cell opens Help. While Help shows, P7 and P8 select the previous and next Help tab. P9 and P10 scroll Help or Messages, or move the selected control on the Controller tab. Back closes Help. A question, text line or learning mode retains higher priority. Help controls must appear in Help itself. A cell used for Help navigation cannot run an action on the screen behind it. The keyboard keeps F1, Tab and its current page keys.

The controller can finish a pending F10 or MIDI-tab learning choice. If no choice is pending, a P7 or P8 press can become the control to learn. With a choice pending, P8 saves it and P7 cancels it without replacing the choice. Back leaves learning. The raw message and its panel-cell event must count as one answer. This rule leaves P7 and P8 available as learned controls.

Exit from the SMC-PAD opens a visible question and does not start quit yet. P8 confirms the question and calls the existing quit path once. P7 or Back cancels it. While the question is open, other panel cells cannot act on the screen. The keyboard keeps its current Ctrl+Q behavior. The Exit cell cannot become a learned MIDI action.

The route inventory adds the following cells to every view. Page 2 holds common actions. Page 3 holds the remaining transport and pattern actions. Each row uses cells 1 through 10 in the order shown. Empty positions stay dim. The existing main and page 1 cells do not move.

| Page | Cells: label (`action`) |
| --- | --- |
| 2 | `outrec` (`record.output`), `trnspt` (`transport.toggle`), `take` (`pad.take`), `proj` (`project.pick`), `save` (`session.save`), `grid` (`ui.pads_toggle`), `learn` (`learn.toggle`), `A1undo` (`pattern.undo`), `A1 ctl` (`pattern.controls`), `exit` (`app.quit`) |
| 3 | `bpm -` (`transport.tempo_down`), `bpm +` (`transport.tempo_up`), `bpm-.1` (`transport.tempo_down_fine`), `bpm+.1` (`transport.tempo_up_fine`), `erase` (`pattern.erase`), `A1 rec` (`pattern.record`) |

Pads adds page 4: `view -` (`pattern.view_prev`), `view +` (`pattern.view_next`), `rows` (`pattern.overview_empty_toggle`) and `step` (`pattern.step_edit`) in cells 1 through 4. Chop and Chop edit add page 4: `hits -` (`chop.hits_back`), `delcut` (`marker.delete`), `delend` (`slice.end_delete`) and `order` (`chop.assign_order`) in cells 1 through 4. A cell that has no effect in its current screen or mode stays visible but does not claim an effective route in the inventory.

The global profile gives first-bank encoder 7 the existing `transport.tempo` step target and encoder 8 the existing `bank` step target. Encoder 6 uses the existing `edit.page` step target. Its physical control identity makes it panel navigation while the panel is visible and leaves it unassigned while hidden. Thus these routes add no public action or MIDI-learn item. Keep the context cell limit, fixed bottom row and existing owner override rule. A mode that cannot show its command has a recorded gap, not a silent fallback.

### PAD BANK shortcut

A capture on 2026-11-18 showed no unsolicited PAD BANK event on the three MIDI inputs. A read of 12 status bytes on the USB Private port showed a pulse in the last byte. The owner tapped PAD BANK six times. With a 52 ms poll interval, six runs of `01` appeared between runs of `00`. Each tap gave one or two `01` replies. This is one run on the supported preset. The last byte signals a press and does not identify the selected bank.

Add `SMC-PAD PAD BANK` to F12 MIDI for a connected USB SMC-PAD with a supported preset. The values are `Off` and `Open panel`. Save the owner choice. Use `Off` when the setting is absent. Use the existing read-only status request on the USB Private port when the shortcut is on. Keep one poll in flight. Route a new press to the Master connection for the same controller.

After enable or reconnect, arm on a valid `00`. Use the next valid `01` to request panel entry once. Require another `00` before a new request. Opening the panel does not run a cell. If the panel is already open, keep its current page. A held Gate keeps its original release. A first-bank hit still closes the panel. Ignore a reply that is older than that hit. An input overlay keeps priority. The first-hit reveal remains available if the shortcut is off or ptsr misses a tap.

## Risks and edge cases

| Situation | Expected result | Status |
| --- | --- | --- |
| A panel cell opens a question, then P8 arrives before redraw | P8 answers once. The first cell cannot answer its own question. | Covered by `modal_answer_wins_over_panel_entry`. |
| A picker opens while the panel is hidden | P8 confirms the picked item on the first hit. No reveal hit runs. | Covered by `modal_answer_wins_over_panel_entry`. |
| A project save fails during SMC confirmation | The current project and picker selection remain. No target opens. | Covered by `refused_project_save_keeps_smc_selection`. |
| A folder picker is open when a Settings knob moves | Only the folder selection changes. The setting stays unchanged. | Covered by `settings_knob_changes_only_the_selected_value`. |
| Help is open when a panel cell arrives | Only the documented Help navigation acts. The underlying screen stays unchanged. | Covered by `help_cells_navigate_without_acting_behind_help`. |
| A Gate pad is held across a picker, question or Help | Its original release stops that pad once. | Covered by `modal_answer_wins_over_panel_entry` and existing Gate tests. |
| Another MIDI controller acts while the SMC panel shows | It keeps its own action and does not select an SMC page. | Covered by `encoder_selects_a_later_context_page`. |
| The SMC-PAD disconnects inside a picker | The picker remains for the keyboard. Panel state clears. No choice is accepted. | Covered by `disconnected_smc_keeps_the_project_picker_for_keyboard`. |
| A silent controller action arrives during the splash | The action can run, and the splash remains until accepted sound. | Covered by existing sound-first splash tests. |
| Exit opens while a recording or save is pending | The question has no quit effect. P8 enters the existing exit-save path once. P7 keeps the session open. | Covered by `exit_question_keeps_a_pending_a1_save_until_confirmation`. |
| An owner binding replaces a panel cell | The owner binding acts outside a question. The reach report names the shipped profile, not the owner's override. | Covered by existing owner-override tests. |
| An owner binding replaces `Alt+U` or sets it to `none` | The owner entry keeps priority. The shipped route test uses the default map and a separate test checks the override. | Add a test in 93. |
| A text line or question is open when `Alt+U` arrives | The active modal route keeps priority. The transport does not toggle behind it. | Add a test in 93. |
| A held Gate or a repeated Alt-arrow key crosses silent pad selection | The Gate keeps its original release target. Only the first key press selects a neighbor. | Add a test in 93. |
| An owner binding or input overlay takes an Alt-arrow key | The owner entry or overlay keeps priority. Chop keeps its slice-bound key. | Add a test in 93. |
| The Chop threshold reaches -40 or 0 dB | A further step keeps the threshold and cuts. It adds no Undo entry. | Covered by `keyboard_and_encoder_set_the_same_chop_threshold`. |
| A threshold key arrives during Chop assign or an overlay | The active mode keeps priority. No threshold recut occurs behind it. | Covered by `threshold_keys_respect_owner_bindings_and_input_owners`. |
| An owner map replaces a threshold key | The owner action or `none` wins over the shipped step. | Covered by `threshold_keys_respect_owner_bindings_and_input_owners`. |

| PAD BANK gives consecutive high replies or a high reply on connect | Open only after a valid low value and one rising edge. Keep an open page in place. | Add a recorded status test. |
| A question, picker, Help, text entry or learning owns input during the pulse | Keep the overlay and target. Do not reveal or activate a cell behind it. | Add an input priority test. |
| A second-bank hit arrives before the status reply | The hit reveals the panel without an action. The late reply keeps the open page. | Add an input order test. |
| A first-bank pad plays before a late reply | The hit closes the panel. The late reply cannot reopen it. The original release stops the same pad. | Add an input order and Gate test. |
| A status reply cannot be tied to the current Master connection | Discard the reply. Reconnect starts hidden. Other controllers keep their actions. | Add a source and reconnect test. |
| The USB request times out or the setting is off | First-hit reveal still works. No reply can run a cell. | Add a fallback test. |

## Open questions

- Which free encoder selects overflow context pages? Default: first-bank encoder 6. It has no shipped role in the current SMC-PAD profile. Keep encoder 1 on the current item and keep both knob banks active.
- Should a silent controller command end the splash? Default: no. The accepted screen-layout specification requires an accepted sound, and silent commands already work while the splash shows.

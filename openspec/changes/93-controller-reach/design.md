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
| Chop | Hits Back and both cut-deletion commands have no shipped SMC-PAD cell. | Open in 93. |

The reach test must decide whether another keyboard action has an equivalent knob, button or context route before it closes this list. A static comparison of action names cannot decide that question.

`ActionInfo` does not yet mark text entry. Add that metadata for `pad.rename` and `browser.filter`. A typed new-project name and keymap edits are document operations, not `ActionInfo` entries. `app.quit` stays outside `ACTIONS` and MIDI learn. The SMC-PAD panel offers Exit as an explicit context cell. No other action gets a text exception. The inventory separates a parameter turn from a button action and names equivalent routes when their action IDs differ.

The story table counts physical presses from the visible main panel and from a hidden panel. A visible context action takes More and its cell. Knob turns do not add presses. The first entry adds PAD BANK and a reveal hit. A target pad hit or a later modal answer is recorded separately. A story that only describes visual state has zero command presses. An X-Session setup story names that device and is not presented as an SMC-PAD action.

### Context pages and modal controls

The current main page and fixed bottom cells remain. More opens the context page. If the inventory needs more context pages, the unused first-bank encoder 6 selects a context page while the SMC panel is visible. The selected page and its cell labels update before the next hit. This keeps a context action within More plus one cell hit from the visible main panel. With the panel hidden, encoder 6 retains its existing unassigned result. The keyboard keeps its direct action paths. This page-selection gesture is navigation within the SMC panel, like PAD BANK selection, and does not change a sampler parameter.

The relative-step event currently omits its source device. Add the source port and control to that event before it drives modal selection or panel page selection. Only the SMC-PAD Master port that owns the visible panel can change its context page. The profile gives encoder 6 a panel-page target, not an edit-page target. A second device keeps its own route. Encoder 1 selects the active project or folder in a picker before it changes the screen behind that picker. Encoder 2 changes a Settings value only without a picker.

Modal handlers take priority over the underlying panel. In a project picker, encoder 1 moves the project selection, P8 confirms and P7 or Back cancels. In the Paths folder picker, encoder 1 moves the row. LEFT and RIGHT move to parent and child folders. P8 selects the current folder, and P7 or Back cancels. A visible question still takes P7 and P8 before either picker. The first-hit reveal rule does not consume a modal answer. A failed project save keeps the picker and selection. A held first-bank Gate receives its original release.

In Settings, encoder 2 changes the selected value. It uses the same action and bounds as the keyboard value keys. When a folder picker opens, the encoder moves only the picker and does not change a setting. The current screen's other knobs keep their roles. The input strip names encoder 2's Settings role.

The main Help cell opens Help. While Help shows, P7 and P8 select the previous and next Help tab. P9 and P10 scroll Help or Messages, or move the selected control on the Controller tab. Back closes Help. A question, text line or learning mode retains higher priority. Help controls must appear in Help itself. A cell used for Help navigation cannot run an action on the screen behind it. The keyboard keeps F1, Tab and its current page keys.

Exit from the SMC-PAD opens a visible question and does not start quit yet. P8 confirms the question and calls the existing quit path once. P7 or Back cancels it. While the question is open, other panel cells cannot act on the screen. The keyboard keeps its current Ctrl+Q behavior. The Exit cell cannot become a learned MIDI action.

The route inventory adds the following cells to every view. Page 2 holds common actions. Page 3 holds the remaining transport and pattern actions. Each row uses cells 1 through 10 in the order shown. Empty positions stay dim. The existing main and page 1 cells do not move.

| Page | Cells: label (`action`) |
| --- | --- |
| 2 | `outrec` (`record.output`), `trnspt` (`transport.toggle`), dim cell 3, `proj` (`project.pick`), `save` (`session.save`), `grid` (`ui.pads_toggle`), `learn` (`learn.toggle`), `A1undo` (`pattern.undo`), `A1 ctl` (`pattern.controls`), `exit` (`app.quit`) |
| 3 | `bpm -` (`transport.tempo_down`), `bpm +` (`transport.tempo_up`), `bpm-.1` (`transport.tempo_down_fine`), `bpm+.1` (`transport.tempo_up_fine`), `erase` (`pattern.erase`), `A1 rec` (`pattern.record`) |

Pads adds page 4: `view -` (`pattern.view_prev`), `view +` (`pattern.view_next`), `rows` (`pattern.overview_empty_toggle`) and `step` (`pattern.step_edit`) in cells 1 through 4. Chop and Chop edit add page 4: `hits -` (`chop.hits_back`), `delcut` (`marker.delete`), `delend` (`slice.end_delete`) and `order` (`chop.assign_order`) in cells 1 through 4. A cell that has no effect in its current screen or mode stays visible but does not claim an effective route in the inventory.

The global profile gives first-bank encoder 7 the existing `transport.tempo` step target and encoder 8 the existing `bank` step target. Encoder 6 uses the existing `edit.page` step target. Its physical control identity makes it panel navigation while the panel is visible and leaves it unassigned while hidden. Thus these routes add no public action or MIDI-learn item. Keep the context cell limit, fixed bottom row and existing owner override rule. A mode that cannot show its command has a recorded gap, not a silent fallback.

## Risks and edge cases

| Situation | Expected result | Status |
| --- | --- | --- |
| A panel cell opens a question, then P8 arrives before redraw | P8 answers once. The first cell cannot answer its own question. | Required recorded MIDI test. |
| A picker opens while the panel is hidden | P8 confirms the picked item on the first hit. No reveal hit runs. | Required keyboard and MIDI test. |
| A project save fails during SMC confirmation | The current project and picker selection remain. No target opens. | Required keyboard and MIDI test. |
| A folder picker is open when a Settings knob moves | Only the folder selection changes. The setting stays unchanged. | Required recorded MIDI test. |
| Help is open when a panel cell arrives | Only the documented Help navigation acts. The underlying screen stays unchanged. | Required recorded MIDI test. |
| A Gate pad is held across a picker, question or Help | Its original release stops that pad once. | Preserve existing audio test. |
| Another MIDI controller acts while the SMC panel shows | It keeps its own action and does not select an SMC page. | Required two-device test. |
| The SMC-PAD disconnects inside a picker | The picker remains for the keyboard. Panel state clears. No choice is accepted. | Required disconnect test. |
| A silent controller action arrives during the splash | The action can run, and the splash remains until accepted sound. | Preserve existing splash tests. |
| Exit opens while a recording or save is pending | The question has no quit effect. P8 enters the existing exit-save path once. P7 keeps the session open. | Required keyboard and MIDI test. |
| An owner binding replaces a panel cell | The owner binding acts outside a question. The reach report names the shipped profile, not the owner's override. | Preserve existing override tests. |

## Open questions

- Which free encoder selects overflow context pages? Default: first-bank encoder 6. It has no shipped role in the current SMC-PAD profile. Keep encoder 1 on the current item and keep both knob banks active.
- Should a silent controller command end the splash? Default: no. The accepted screen-layout specification requires an accepted sound, and silent commands already work while the splash shows.

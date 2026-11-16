# Design

## Context and evidence

Change 92 supplies the panel and its first-hit rule. `config/panel.toml` gives one context page per view. In `crates/app/src/dispatch.rs`, `PanelCell` accepts only Back while a picker or Help is open. The project-list step target exists, but the shipped SMC-PAD profile does not bind it in those modes. `crates/input/src/lib.rs` excludes `app.quit` from `ACTIONS` so that MIDI learn cannot offer it. Keep that safety rule.

`openspec/specs/screen-layout/spec.md` says that sound, not a silent input, ends the splash. The planned suggestion to close it on any controller input conflicts with this accepted requirement. The splash does not block commands. Change 93 keeps the current splash rule and updates the plan.

## Decisions

### Reach inventory

The reach inventory starts with effective keyboard actions on each screen and in each mode. It also includes panel cells, profile buttons and knobs that have no direct keyboard binding. Tests use the shipped `config/keymap.toml`, `config/devices/smc-pad.toml` and `config/panel.toml`, then send recorded keyboard events and raw MIDI through production dispatch. An action name in a TOML file alone is not evidence that a modal handler accepts it. The test observes the result and the mode badge.

`ActionInfo` marks `pad.rename` and `browser.filter` as text entry. A typed new-project name and keymap edits are document operations, not `ActionInfo` entries. `app.quit` stays outside `ACTIONS` and stays keyboard-only. No other action gets a text exception. The inventory separates a parameter turn from a button action and names equivalent routes when their action IDs differ.

The story table counts physical presses from the visible main panel and from a hidden panel. A visible context action takes More and its cell. Knob turns do not add presses. The first entry adds PAD BANK and a reveal hit. A target pad hit is recorded separately because it selects a sound or target. A story that only describes visual state has zero command presses. An X-Session setup story names that device and is not presented as an SMC-PAD action.

### Context pages and modal controls

The current main page and fixed bottom cells remain. More opens the context page. If the inventory needs more context pages, the unused first-bank encoder 6 selects a context page while the SMC panel is visible. The selected page and its cell labels update before the next hit. This keeps a context action within More plus one cell hit from the visible main panel. With the panel hidden, encoder 6 retains its existing unassigned result. The keyboard keeps its direct action paths. This page-selection gesture is navigation within the SMC panel, like PAD BANK selection, and does not change a sampler parameter.

Modal handlers take priority over the underlying panel. In a project picker, encoder 1 moves the project selection, P8 confirms and P7 or Back cancels. In the Paths folder picker, encoder 1 moves the row. LEFT and RIGHT move to parent and child folders. P8 selects the current folder, and P7 or Back cancels. A visible question still takes P7 and P8 before either picker. The first-hit reveal rule does not consume a modal answer. A failed project save keeps the picker and selection. A held first-bank Gate receives its original release.

In Settings, encoder 2 changes the selected value. It uses the same action and bounds as the keyboard value keys. When a folder picker opens, the encoder moves only the picker and does not change a setting. The current screen's other knobs keep their roles. The input strip names encoder 2's Settings role.

The main Help cell opens Help. While Help shows, P7 and P8 change its page, P9 and P10 scroll the shown page, and Back closes it. A question, text line or learning mode retains higher priority. Help controls must appear in Help itself. A cell used for Help navigation cannot run an action on the screen behind it. The keyboard keeps F1, Tab and its current page keys.

The exact added context cells follow the reach inventory. Before code, add their labels and action IDs to this design and the active specification. Keep the context cell limit, fixed bottom row and existing owner override rule. A mode that cannot show its command has a recorded gap, not a silent fallback.

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
| An owner binding replaces a panel cell | The owner binding acts outside a question. The reach report names the shipped profile, not the owner's override. | Preserve existing override tests. |

## Open questions

- Which free encoder selects overflow context pages? Default: first-bank encoder 6. It has no shipped role in the current SMC-PAD profile. Keep encoder 1 on the current item and keep both knob banks active.
- Should a silent controller command end the splash? Default: no. The accepted screen-layout specification requires an accepted sound, and silent commands already work while the splash shows.
- Should `app.quit` be available from MIDI? Default: no. The current action inventory deliberately omits it. The owner can leave ptsr with Ctrl+Q.

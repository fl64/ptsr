# Design

## User contract

The owner opens the A1 overview with F5. F7 proposes a separate edit mode in that overview. The overview remains read-only outside that mode. In edit mode, the owner selects a time cell, chooses a pad and removes one existing hit. An empty cell never inserts a hit. The program does not change the stored timing of other hits.

This is the first correction step after the full-bank overview. The older 98a draft placed the editor on Pads and assigned F5 to edit entry. Both choices conflict with the accepted F5 overview. This design moves editing into that overview and keeps F5 unchanged.

## Entry and navigation

Entry needs an existing A1 and no active recording, pending capture, pad edit, pattern controls, placement, marking or audio recording. A refusal names the blocking work and the next action. Entry does not move transport, mark the project dirty or create history. The first entry uses the current playback cell when valid audio feedback exists. Otherwise, it selects the first cell. Reentry in the same project keeps the last edit cell.

F7 toggles edit mode while the overview is open. Esc closes help first, then edit mode, then the overview. F2 closes edit mode and returns to Pads. F5 continues to open the overview. U/O select banks. J/L or Left/Right select time cells. I/K or Up/Down select events in the chosen pad and cell. Alt+Up/Alt+Down keep the shared view division. Ctrl+Z uses common Undo. Learned MIDI buttons use the same logical actions.

The edit cursor stays on its selected time page while A1 plays. The audio snapshot moves a separate playback plate. The view division changes only the projection. On a division change, the cursor selects the cell that contains its old starting tick. A bank change keeps the time cell and changes the selected pad bank.

## Pad input and event identity

Edit pad presses use a silent UI route for keyboard and MIDI. A normal Gate that started before edit entry keeps its original release. An edit press never becomes sound after exit or a bank change. Text, questions, pickers and learning retain their existing priority.

A press on an empty cell selects the pad and changes nothing. A press on a cell with one event removes that event. On a different pad with several events, the first press selects its first event without removal. I/K or Up/Down choose among the events. Another press on that pad removes only the selected event. Repeats and releases cannot remove another event. The UI shows the selected event index and count before removal.

The App uses the existing checked raw rows and compiler descriptors. It filters hidden rows before it projects cells. It keeps separate identities for collisions and same-tick rows. It checks the project, pattern generation, pad, cell and row identity immediately before removal. If one changed, the App refreshes selection without removing a replacement row.

## Publication, history and save

The App builds and checks the replacement before it changes history. It uses the existing phase-preserving publication path. Audio accepts the new pattern at a block boundary. Already-started Gates keep their deadlines and tails. Past onsets do not replay. Queue pressure must not lose or reorder accepted edits. The audio callback gains no allocation, lock, I/O, logging or panic.

Each successful removal adds one common-history Undo step. Existing pattern Undo may restore the newest permitted pattern edit without crossing a newer unrelated edit. Cursor, view and empty presses create no history. Save writes corrected raw rows in format 5. It does not save editor mode or cursor. A save refusal keeps unsaved work and Undo. A project switch closes the editor and resets the view to 1/16.

## Layout and feedback

The existing overview keeps the selected bank's 16 rows at 80x26 and 120x40. Edit mode names `STEP EDIT`, outlines the selected time cell and highlights its bank and pad. The cursor differs from the moving playback plate by shape and background. The bank index, long names, safety notice and three-row help stay visible. The hint line shows the selected event index and count, or explains that the cell is empty. A successful removal names the pad and one-event scope with the effective Undo key. F1 explains both markers and the absence of insertion.

The offline catalogue adds occupied, empty, multiple-event, off-grid, playing and safety states. A generated-sound WAV shows one removal and Undo while another pad and an existing Gate continue. Automatic checks cover timing, clipping, silence and clicks. Hardware, terminal reporting and listening remain owner checks.

## Open questions

- Should the first editor remove only existing hits or also insert and move them? Default: remove one existing hit with Undo. Add insertion and movement in later changes.
- Should F7 toggle edit mode or should a second F5 do so? Default: F7 toggles edit mode. F5 retains one screen action on every screen.
- Should the first editor show raw tick, played tick, velocity and Gate length for each event? Default: show event index and count in the limited 80x26 hint line. Put detailed values in F1 or a later detail panel.

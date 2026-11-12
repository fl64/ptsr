# Plan 98a: step edit for A1 (historical draft)

This file is a guide, not a requirement. It does not authorize code.
Change 98 now covers only the rhythm view. The owner accepted that view on 2026-11-11.
The step editor below was part of the first 98 proposal. It moves here so that change 98 can close with complete tests.
The owner chose a full-bank A1 overview first on 2026-11-12. Change 98l adds that read-only screen.
Change 98a implements one-hit correction after the overview. Its [archived proposal](../../openspec/changes/archive/2026-11-12-98a-step-edit/proposal.md) and the main specifications contain the accepted requirements.
This draft records the earlier plan. Use the specifications for current behavior.

Context: [the archived change 98 design](../../openspec/changes/archive/2026-11-12-98-pattern-screen/design.md) keeps the earlier editor decisions and edge cases.
Its F5 edit key and Pads-grid layout conflict with the accepted 98l overview. The archived 98a change revises those choices.
Change 98 replaced F7/F8 with Alt+Up/Alt+Down for the view. Read the keyboard requirements below with that change.

## patterns

### Draft requirement: Explicit targeted-removal mode

Step edit SHALL require existing A1 on Pads with no incompatible input owner or active or pending recording work. Entry and exit SHALL NOT restart transport or create history. Edit pad presses SHALL remain silent and SHALL NOT enter recording. Outside edit mode, pads SHALL retain ordinary behavior. Empty-cell presses SHALL select only, without insertion or sample removal.

#### Scenario: Pattern edit entry is explicit and preserves playback
- **WHEN** the owner toggles step edit with absent, empty or populated A1 while transport remains stopped or playing
- **THEN** absent A1 refuses with a recording instruction and existing A1 enters or exits without moving the clock
- **AND** first entry selects the current playback cell with valid feedback, or the first cell otherwise, while reentry retains the cursor
- **AND** empty presses and navigation remain silent and do not change raw rows, dirty state or history

#### Scenario: Pattern edit refuses incompatible work
- **WHEN** the owner requests edit entry or incompatible work during capture, recovery, placement, marking, pad edit, pattern controls, audio recording or restoration
- **THEN** the incompatible request refuses with a next-step instruction without ending or changing the existing work
- **AND** text, pickers, questions, learning and original releases keep their existing priority and ownership

### Draft requirement: Remove exactly one identified pattern event

A unique-event pad press SHALL remove only that event in the selected cell. For a different pad with multiple events, the first press SHALL select its first event without removal. Event selection SHALL expose the chosen index and count. A subsequent selected-pad press SHALL remove only that chosen raw row. Repeats and releases SHALL NOT remove another event. Invalidated identities SHALL refresh without removing a replacement row.

#### Scenario: Pattern edit removes one event and preserves other sounds
- **WHEN** the owner presses a lit pad with one event in edit mode
- **THEN** only its event in that cell disappears and its sample, other events, other pads, settings and held-note tails remain unchanged
- **AND** the press, its repeats and its release produce no sound or additional removal

#### Scenario: Pattern edit selects within an ambiguous cell
- **WHEN** another pad has several events in the selected cell and the owner selects that pad, chooses an event and presses it again
- **THEN** the first press removes nothing and the following pad press removes only the chosen identified row
- **AND** remaining rows, including same-tick collisions, retain their timing, data and visible count

#### Scenario: Pattern edit cannot remove a stale replacement row
- **WHEN** the pattern or project changes after event selection but before removal
- **THEN** an invalid old selection cannot remove a new row that reuses its index or identity slot
- **AND** deferred old-project input cannot change the current pattern

### Draft requirement: Pattern removal preserves ordered publication

App SHALL check a replacement before mutation and use existing phase-preserving publication at an audio boundary. Transport, metronome and active note deadlines SHALL remain unchanged. Past onsets SHALL NOT replay. Queue or retirement pressure SHALL retain whole ordered edits without callback allocation, destruction, blocking or late-hit bursts. Already-started notes SHALL finish under their existing rules.

#### Scenario: Pattern edit publication survives pressure without retriggering
- **WHEN** one removal and another ordered edit encounter command or retirement pressure while A1 and Gate notes play
- **THEN** accepted replacements remain whole and ordered, the clock continues and no past onset repeats
- **AND** existing note deadlines and tails remain valid without callback allocation, deallocation or blocking

### Draft requirement: Pattern removal has one-event Undo and existing save safety

Each successful removal SHALL create one common-history Undo step. Common Undo and existing pattern Undo SHALL restore the newest permitted pattern edit without crossing a newer unrelated edit. View and cursor changes SHALL create no history. Save SHALL retain corrected raw rows through format 5 without storing editor state. Write refusal SHALL preserve unsaved work and Undo. Load SHALL restore stopped transport and closed edit mode.

#### Scenario: Pattern removal Undo restores one edit
- **WHEN** the owner removes two events and uses common or pattern Undo
- **THEN** each Undo restores only the newest permitted removal and its exact row data
- **AND** a newer pad or FX edit retains existing history ordering and no redo becomes available

#### Scenario: Pattern correction saves without editor state
- **WHEN** a correction encounters save refusal, then saves successfully and reopens
- **THEN** refusal retains the correction and Undo, while the reopened format-5 data contains exactly the corrected raw rows and unchanged settings
- **AND** load keeps transport stopped and edit mode closed, while the view returns to 1/16

## keyboard

### Draft requirement (MODIFIED): Command key roles across screens

I/K and Up/Down SHALL move selection, row or zoom. Enter SHALL be the main action, Esc back, Backspace removal, M marking or cutting, and bound Tab/Shift+Tab next/previous page or group. On Pads and FX, J/L and Left/Right SHALL change the selected parameter with edit open or closed. During explicit Pads step edit, horizontal actions SHALL instead select the time cell. Vertical actions SHALL select its event. Page actions SHALL select the view division. A bound Ctrl+D SHALL mean duplicate or double.

#### Scenario: Shift H is a distinct command in both input forms
- **WHEN** Chop receives H and Shift+H as extended key events or legacy lowercase and uppercase characters in either keyboard layout
- **THEN** H resolves to `chop.hits` and Shift+H to `chop.hits_back`
- **AND** Shift+Tab, existing Ctrl/Alt chords and full-velocity pads keep their behavior

## ADDED Requirements

### Draft requirement: Pattern step editor keys

Pads SHALL bind F5 to `pattern.step_edit`, Alt+Down to `pattern.view_prev` and Alt+Up to `pattern.view_next`. F5 SHALL toggle the mode, not open another screen. In step edit, effective horizontal, vertical and page actions SHALL navigate steps, events and views respectively. F5 or unclaimed Esc SHALL close the mode. Outside that mode, existing parameter and page actions SHALL retain their roles. F1 SHALL suspend editor navigation while its own controls apply.

#### Scenario: Pattern editor keys have explicit contextual roles
- **WHEN** the owner uses default F5, J/L, arrows, I/K, Tab/Shift+Tab, Alt+Up/Alt+Down and Esc on Pads
- **THEN** edit mode shows the applicable step, event and view changes without changing pad parameters
- **AND** outside edit mode the existing direction and page roles remain, while Alt+Up/Alt+Down change only the view

### Draft requirement: Step editor preserves original pad release ownership

A press accepted for edit SHALL NOT sound after a later mode change or delayed delivery. A press accepted for ordinary playing SHALL retain its original release across editor, screen and bank changes. Input priority SHALL remain source-specific for text, pickers, questions and learning. Editor actions SHALL NOT bypass those owners.

#### Scenario: Pattern editor mode changes retain held-note ownership
- **WHEN** keyboard or controller pad holds cross step entry, exit, bank or screen changes
- **THEN** an earlier sounding Gate releases its original voice and an edit press remains silent after exit
- **AND** old releases, repeats and delayed notifications cannot remove another event or start a new-bank sound

## midi-learn

### Draft requirement: Learn pattern step editor commands

F12 MIDI SHALL list `pattern.step_edit`, `pattern.step_prev`, `pattern.step_next`, `pattern.view_prev`, `pattern.view_next`, `pattern.event_prev` and `pattern.event_next` as GLOBAL button actions. Commands SHALL use existing confirmation, clearing, persistence and layer precedence. They SHALL act only on applicable Pads state without changing screens. Unavailable commands SHALL explain the required screen or mode. No factory device binding SHALL be introduced.

#### Scenario: Learned pattern editor buttons persist without profile changes
- **WHEN** the owner learns separate note or CC buttons for the editor commands, confirms them, saves and reloads
- **THEN** their actions remain available without resetting existing assignments or X-Session defaults
- **AND** learning selection, cancellation and release do not execute an editor command

### Draft requirement: Pattern edit commands precede following pad input

A learned mode or cursor command SHALL take effect before a following pad event in the same ordered production stream. Edit pad onsets SHALL reach App silently with their original absolute pad, source and project ownership. Releases SHALL retain their original owner. Stale project input and invalid shared focus SHALL NOT edit the current pattern or become sound.

#### Scenario: Raw MIDI pattern editor commands precede removal pads
- **WHEN** raw profile-routed MIDI enters edit mode, selects a step and presses its pad before a redraw
- **THEN** the actions follow that order and the press addresses only the selected cell without immediate audio or recording
- **AND** its release cannot repeat removal or become a sounding pad

#### Scenario: Raw MIDI pattern editor rejects stale work
- **WHEN** a mode change, bank change, project transition or invalid focus delays controller input
- **THEN** old edit presses cannot remove replacement-project rows or sound after edit exit
- **AND** original sounding-note releases retain their permitted ownership

### Draft requirement: Pattern editor input paths have behavioral parity

The same event selection, view change, targeted removal and Undo SHALL work through keyboard and raw MIDI profile routing. Physical velocity SHALL NOT change removal choice. Editor commands SHALL use the same App transitions, not simulated keyboard input or a test-only route. Physical-controller and SMC-PAD factory acceptance SHALL remain separate from offline tests.

#### Scenario: Pattern correction matches through keyboard and raw MIDI
- **WHEN** the owner corrects the same recorded pattern through keyboard and learned raw MIDI commands, including ambiguous cells and Undo
- **THEN** both runs retain identical raw rows, settings, history ordering and silent edit behavior
- **AND** subsequent ordinary pad presses retain keyboard Shift and actual MIDI velocity behavior

## screen-help

### Draft requirement: Pattern correction help follows the active pad role

Ordinary Pads help SHALL expose the effective step-editor entry and view controls when A1 exists. Step edit SHALL prioritize step movement, pad removal, event selection, view and exit. F1 SHALL include the complete command list and both marker meanings. Existing modifier filtering, three-row geometry, owner overrides and `none` SHALL remain effective. Editor help SHALL NOT describe pad presses as live playing or insertion.

#### Scenario: Pattern correction help exposes the next action
- **WHEN** the UI shows ordinary pattern view, step edit, an empty cell or a multiple-event cell at either supported size
- **THEN** help names the applicable entry, selection, removal or exit action through effective bindings
- **AND** F1 explains off-grid markers, both cursors, event multiplicity and the absence of insertion

### Draft requirement: Pattern correction result explains its Undo scope

A successful removal SHALL report the affected pad and one-event scope with the effective common-Undo control. Full help for pattern Undo SHALL distinguish one recording pass from one targeted removal, according to history. No-op presses and ambiguous-pad focus SHALL NOT claim removal. Save failure and audio warnings SHALL retain existing priority.

#### Scenario: Pattern correction feedback distinguishes focus removal and Undo
- **WHEN** an empty press, ambiguous-pad focus, one removal and its Undo occur in sequence
- **THEN** only actual changes report removal or restoration and their scope is one selected event
- **AND** result text names the effective Undo binding without implying removal of an earlier recording pass

### Draft requirement: Pattern editor refusals explain a safe next step

Refusal SHALL identify absent A1, incompatible mode, pending capture, unavailable event identity or required Pads context as applicable. It SHALL preserve existing work and give an effective recovery action. F1 SHALL suspend event removal until closed without converting edit presses into live sound. Closing it SHALL retain editor selection.

#### Scenario: Pattern correction refusal and help preserve work
- **WHEN** another input owner, stale selection or absent A1 blocks edit actions, or the owner opens F1 during editing
- **THEN** the current work remains unchanged and refusal or help identifies the safe next action
- **AND** closing F1 restores selection without a removal or delayed sounding hit

## screen-layout

### Draft requirement: Pattern playback and edit positions are independent

The playback marker SHALL use coherent current-project audio position, never a UI timer. It SHALL be a centered three-column light plate in both event rows, without an arrow, frame or extra row. Step edit SHALL keep its own cursor and page during playback. The two markers SHALL differ by more than color. Off-page playback SHALL show its position as text only. Stale or missing snapshots SHALL NOT fabricate motion or show an old project's position.

#### Scenario: Pattern edit cursor does not follow playback
- **WHEN** playback crosses beats, pages and loop seams while an edit cursor remains on another page
- **THEN** the edit cursor and visible page stay selected and playback retains a separately identified position
- **AND** missing or old-project snapshots do not advance or replace that marker

### Draft requirement: Pattern edit has a distinct pad role

Step edit SHALL show `STEP EDIT`, yellow mode frames and distinct onset-presence markers for pads at the selected cell. The markers SHALL NOT claim sounding voices. Ordinary selected-pad identity and bank SHALL remain visible. Entry and exit SHALL clear only editor markers. Step edit SHALL temporarily show the pad grid when its saved preference is hidden. Exit SHALL restore that preference without saving a visibility change.

#### Scenario: Pattern edit pads distinguish events from sound
- **WHEN** step edit selects occupied and empty cells across banks, including a Gate sustained from an earlier cell
- **THEN** pad markers show onsets at the selected cell rather than every currently sounding note
- **AND** exit restores ordinary pad styling and visibility without a sounding-hit flash from an edit press

### Draft requirement: Pattern editor catalogue states

The offline catalogue SHALL cover ordinary pattern view and step edit at both supported sizes. It SHALL include straight and triplet pages, off-grid and multiple events, independent cursors, hidden-grid preference and safety-warning combinations. Missing required entries SHALL fail matrix coverage. Generation SHALL NOT open audio or MIDI devices or edit projects.

#### Scenario: Catalogue covers pattern view and step editing
- **WHEN** the offline catalogue generates after editor implementation
- **THEN** both sizes contain the required pattern-view and edit combinations and missing entries fail coverage
- **AND** generation leaves projects and device access untouched without claiming actual kitty or physical-controller acceptance


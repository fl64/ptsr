# screen-help Delta

## MODIFIED Requirements

### Requirement: Recording help follows effective keys

Default recording help SHALL name Ctrl+R pattern recording, Alt+R microphone hold and Ctrl+T output. Without releases, microphone help SHALL name toggle. Owner recording keys SHALL override defaults in help. Pattern recording SHALL remain distinct from audio capture. No help SHALL advertise an unimplemented SMC-PAD hold or factory role.

#### Scenario: Microphone help follows release support
- **WHEN** the owner views microphone help or records with the default key
- **THEN** the key is Alt+R, with hold/release instructions when supported and toggle instructions otherwise

#### Scenario: Help respects an owner recording key
- **WHEN** the owner explicitly maps Ctrl+R to microphone recording
- **THEN** help names that effective key and does not claim it only shows a move notice or records A1

#### Scenario: Full help distinguishes the three recording targets
- **WHEN** F1 opens with default keys on each current screen
- **THEN** Ctrl+R names A1 recording, Alt+R names microphone and Ctrl+T names output
- **AND** owner overrides and `none` change those labels without an obsolete move notice

## ADDED Requirements

### Requirement: Full help remains readable across pages

F1 SHALL wrap long keys and descriptions without lost text at 80x26 and 120x40 on every current screen.
The current position and effective page keys SHALL remain visible.
Tabs SHALL retain independent offsets. Reopening SHALL start at the first key-table row.
Resize and content changes SHALL clamp to the last full viewport.
The frame's accepted recording caption SHALL remain visible on either tab.
Fixed safety rows, compact-help rows and audio behavior SHALL remain unchanged.

#### Scenario: Every full help row is reachable
- **WHEN** the owner pages through F1 on each current screen at either supported size
- **THEN** long keys, descriptions, recording instructions and cross-bank erasure identities remain readable without lost text
- **AND** the visible page controls follow effective bindings rather than disabled defaults
- **AND** ordinary modifier help does not advertise commands that require the F1 key table

#### Scenario: Full help retains accepted recording feedback
- **WHEN** accepted CountIn, Recording, erasure, Finishing or Idle changes while either F1 tab is open
- **THEN** the visible caption follows that accepted phase and position without a premature queued state
- **AND** the complete target list remains reachable without hiding fixed safety rows

#### Scenario: Help page offsets follow local lifecycle
- **WHEN** the owner changes help tabs, resizes, changes the key-table content or reopens F1
- **THEN** tab offsets remain independent, content changes clamp to a valid viewport and reopening starts at the first key-table row
- **AND** the underlying screen, bank, selection and recording state remain unchanged by page navigation

### Requirement: Bound modifier help includes zoom

Chop Ctrl and Alt help SHALL name effective I/K zoom controls beside their existing J/L bound controls.
F1 SHALL include the effective modified zoom bindings.
Removed or rebound keys SHALL NOT appear as zoom controls.

#### Scenario: Held bound modifiers show working zoom keys
- **WHEN** Chop shows Ctrl, Alt or full help with default or overridden zoom bindings
- **THEN** the help identifies working zoom and bound keys without changing their actions
- **AND** an explicit owner removal or different action changes the shown zoom controls

### Requirement: Pitch Gate help separates settings from sound

Pads help SHALL state that Pitch Gate acts only on A1 notes, not manual hits or previews. On but inapplicable feedback SHALL explain the source or pitch restriction in text. F1 SHALL explain linked slices, positive pitch and the next-grid cut. Toggle popups SHALL retain pad identity, saved state and applicability. Help SHALL use effective keyboard and learned-button controls without new bindings or audio behavior.

#### Scenario: Pitch Gate help explains an inaudible manual toggle
- **WHEN** keyboard or raw MIDI toggles Pitch Gate while the owner plays manual pads with edit open or closed
- **THEN** visible help explains unchanged manual sound and the popup identifies the pad, saved setting and applicability
- **AND** F1 explains A1-only linked-slice positive-pitch cuts with effective controls and the source or pitch reason when inapplicable

### Requirement: Pattern help names the current next step

Count-in help SHALL name cancellation and state that pad hits are not recorded yet. With A1 and no active pass, help SHALL name recording and pass Undo. Recording help SHALL name end, erasure and pass Undo. Erasure help SHALL name silent pad targets and the return to playing. Pattern controls SHALL name selection, value changes and closing. F1 SHALL include actions omitted from compact help. Existing effective-key, modifier and fixed three-row rules SHALL remain.

#### Scenario: Recording help changes with the accepted phase
- **WHEN** count-in, recording, erasure and pattern controls appear in separate fixtures
- **THEN** help names the working next step and effective keys for that phase
- **AND** closing a panel or erasure returns to the applicable previous help without moving screen rows

#### Scenario: Count-in and idle help name hidden rules
- **WHEN** count-in runs, then the pass ends and A1 stays without an active pass
- **THEN** count-in help says that pad hits are not recorded yet, and idle help names recording and pass Undo with effective keys
- **AND** a `none` binding for pass Undo removes that item without a fallback

### Requirement: A completed pass reports its result

When a pass ends, App SHALL post a Done notice with the count of new hits and of replaced older hits. A pass without new hits SHALL say that it added nothing. The notice SHALL NOT replace save or audio warnings.

#### Scenario: A completed pass reports new and replaced hits
- **WHEN** a pass adds three hits, one of them replaces an older hit, and a later pass adds no hits
- **THEN** the first notice reports three new hits and one replaced hit, and the second reports that the pass added nothing
- **AND** save and audio warnings keep their priority over the notice

### Requirement: Pattern refusals explain recovery

Refused recording, length edits, erasure and pattern Undo SHALL name the actual blocking state and a working next step. Manual-save help SHALL state that saving completes the pass while playback continues. Capture overflow and the raw-hit limit SHALL use Warn notices without replacing persistent save or audio status. No-release erasure help SHALL state one loop per pad press.

#### Scenario: Pattern warnings explain limits without claiming success
- **WHEN** a phase, newer Undo step, full pattern or capture overflow prevents an operation
- **THEN** the notice identifies the actual limit and keeps applicable recovery controls visible
- **AND** it does not show a success result or hide save and audio warnings

#### Scenario: Save and erase fallbacks are stated before use
- **WHEN** F1 shows pattern recording help with or without key-release support
- **THEN** it explains pass completion on manual save and the applicable erase hold or one-loop fallback
- **AND** microphone hold or toggle remains a separate instruction

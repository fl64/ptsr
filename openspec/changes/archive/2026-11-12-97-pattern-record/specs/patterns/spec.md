# patterns Delta

## ADDED Requirements

### Requirement: Start and end a recording pass

`pattern.record` SHALL toggle recording into A1 without changing screen, bank or selected pad. With no pattern, a successful start SHALL create an empty two-bar A1 at 1/16 and 100 percent strength. Starting during playback SHALL preserve clock position. Ending recording SHALL keep playback running. Each accepted start and end SHALL belong to one pass.

#### Scenario: Record a first rhythm without a project file edit
- **WHEN** the owner starts pattern recording with no A1, plays pads and ends recording
- **THEN** A1 contains the accepted manual hits and repeats with the default two-bar settings
- **AND** the screen, selected pad, bank and existing sounds stay unchanged

#### Scenario: Overdub an existing rhythm without restarting it
- **WHEN** the owner starts recording during A1 playback, adds hits and ends recording
- **THEN** previous raw rows remain and the new hits join them without restarting transport
- **AND** another recording toggle starts a separate pass

### Requirement: One bar of count in

Starting recording while stopped SHALL produce four count-in beats over one bar of 4/4. A1 playback and capture SHALL start once at tick zero after that bar. Count-in SHALL use the current tempo and the metronome sound without changing the saved metronome preference. Warm-up pad hits SHALL sound normally without entering the take.

#### Scenario: Count in before the first recorded beat
- **WHEN** recording starts while stopped with metronome preference off
- **THEN** four count-in clicks precede A1 tick zero by exactly one musical bar
- **AND** metronome preference stays off, and a warm-up hit does not enter the take

#### Scenario: Change tempo during count in
- **WHEN** tempo changes during count-in
- **THEN** remaining beats follow the new tempo without a repeated or omitted beat
- **AND** recording starts once after four musical beats

### Requirement: Cancel count in without recording

A second recording toggle or an unclaimed Esc SHALL cancel count-in and leave transport stopped. A newly created empty A1 SHALL remain. Cancellation SHALL add no recorded rows or pass Undo step. Releases from warm-up pads SHALL retain their original sound ownership.

#### Scenario: Cancel count in with a pad held
- **WHEN** a pad sounds during count-in and the owner cancels before tick zero and releases that pad
- **THEN** no recorded hit or pass Undo step appears and transport stays stopped
- **AND** the release reaches its original pad without recording a late event

### Requirement: Transport requests during count in

Transport Stop SHALL cancel count-in and leave playback stopped. Explicit transport Play or a stopped-to-playing transport toggle SHALL cancel count-in and start ordinary playback from zero without capture. Repeated recording starts SHALL NOT restart or shorten an accepted count-in. A mismatched recording end SHALL NOT cancel it.

#### Scenario: Transport requests cannot start recording early
- **WHEN** Stop, Play or a transport toggle arrives during count-in
- **THEN** the count-in ends without captured rows, and only Play or the transport toggle starts ordinary playback
- **AND** later audio cannot resume the cancelled recording pass

#### Scenario: Repeated recording requests preserve the count in
- **WHEN** repeated recording starts or a mismatched recording end arrive during count-in
- **THEN** the accepted pass retains its original four-beat schedule
- **AND** only an end for that pass can cancel it before tick zero

### Requirement: Capture accepted manual pad input

Recording SHALL capture only manual pad commands accepted by ready audio for the current project. It SHALL preserve absolute pad and MIDI velocity. Sequencer hits, previews, clicks, blocked input and derived notifications SHALL NOT enter the take. Accepted empty or muted pads SHALL remain recordable without claiming audible playback.

#### Scenario: Record manual pads without recording the sequencer
- **WHEN** keyboard pads, raw MIDI notes and learned pad buttons play over an existing pattern
- **THEN** only the accepted manual onsets enter the take with their absolute pads and velocities
- **AND** pattern repeats, previews, clicks and duplicate notifications add no raw rows

#### Scenario: Refused sound input cannot create a recorded onset
- **WHEN** an unready project or full command queue rejects a manual onset
- **THEN** that onset creates no recorded row and no later replay
- **AND** an accepted empty-pad onset may create a row but no false PLAY indication

### Requirement: Recorded timing follows accepted audio

Live recording SHALL timestamp manual input at its accepted audio-block start. Input timing uncertainty SHALL remain at most one actual audio buffer. Raw ticks SHALL round to the nearest integer, with ties later. Playback of a completed take SHALL retain the existing one-output-frame scheduling accuracy, independent of redraw timing.

#### Scenario: Recorded positions follow audio blocks rather than redraws
- **WHEN** identical accepted input renders with different UI delays and known audio-block boundaries
- **THEN** raw ticks match those audio boundaries and later playback follows the raw or quantized ticks within one output frame
- **AND** the result does not claim sub-buffer live input accuracy

### Requirement: Fresh hits do not play twice

A manual hit SHALL sound immediately without another same-lap trigger from its new row. Its first sequenced occurrence SHALL be in a later eligible loop. A quantized seam hit SHALL become eligible at the next loop start. Delayed publication SHALL NOT replay past occurrences. Recompiling settings SHALL NOT repeat an already played occurrence.

#### Scenario: A hit before the grid sounds once in its capture lap
- **WHEN** a manual hit occurs 20 ms before a grid line and recording applies 100 percent quantize
- **THEN** it sounds once immediately, not again on that same-lap grid line
- **AND** its later loop occurrence plays on the grid within one output frame

#### Scenario: A recorded seam hit first repeats at the next loop start
- **WHEN** a late-loop manual hit quantizes to the loop end and wraps to zero
- **THEN** it sounds once manually and first repeats at the next eligible zero tick
- **AND** it neither repeats before that seam nor skips an extra complete loop

#### Scenario: Delayed capture publication does not produce a catch up burst
- **WHEN** UI delay postpones a pattern update beyond one or more eligible occurrences
- **THEN** elapsed occurrences stay missed and only future eligible occurrences play
- **AND** grid changes and additional capture updates do not repeat consumed occurrences

### Requirement: Recording preserves eligible collision hits

At a shared pad and compiled tick, playback SHALL select only eligible rows. An ineligible fresh row SHALL NOT suppress an eligible old row. Eligible collisions SHALL retain the existing order: higher velocity, longer gate, then earlier raw row. Playback SHALL start at most one sequenced hit per collision without merging raw rows. Manual hits SHALL remain independent of old sequenced hits.

#### Scenario: An ineligible overdub keeps the previous rhythm
- **WHEN** a stronger fresh hit shares a compiled pad and tick with an eligible old hit
- **THEN** the old hit plays once while the fresh hit remains ineligible
- **AND** the stronger hit wins once eligible, without deleting either raw row while the pass records

#### Scenario: A manual hit does not suppress an existing pattern hit
- **WHEN** a manual hit sounds 20 ms before an existing same-pad pattern hit during recording
- **THEN** the manual hit and the old pattern hit sound at their respective accepted times
- **AND** the fresh row adds no same-lap sequenced trigger

### Requirement: A completed pass replaces same-step hits

When a pass completes, each fresh row SHALL replace older rows of the same absolute pad at the same compiled playback tick. The pass's final quantize and strength SHALL set that tick. The most recently captured row SHALL stay, also between two fresh rows of one pass. Other ticks, other pads and rows hidden by the loop length SHALL stay. Replacement SHALL NOT occur while the pass records. Pass Undo SHALL restore replaced rows. Load SHALL NOT remove rows.

#### Scenario: A replayed step keeps only the newest hit
- **WHEN** a pass adds a hit on a pad whose older row compiles to the same tick, and the pass ends
- **THEN** only the newest row remains at that pad and tick, with its velocity and Gate length
- **AND** later loops play that row once

#### Scenario: Replacement keeps other steps, pads and hidden rows
- **WHEN** a completed pass adds hits next to older rows at other ticks, on other pads and beyond a shortened loop
- **THEN** all those older rows remain, and quantize Off keeps hits at different ticks separate
- **AND** a loaded project keeps its saved same-tick rows until a pass replaces them

#### Scenario: Undo restores replaced hits
- **WHEN** the owner undoes a completed pass that replaced older rows
- **THEN** the older rows return with their original order, velocity and Gate length
- **AND** the fresh rows of that pass disappear

### Requirement: Record complete gate durations

A Gate onset SHALL remain pending until release, retrigger or pass end determines its musical duration. Pending hits SHALL NOT replay. Positive durations SHALL use 1–245760 ticks. One-shot hits SHALL use zero gate length. A new onset on one pad SHALL close its previous pending gate. Completion SHALL NOT change another pass or a later press.

#### Scenario: Record a gate across the loop seam and a tempo change
- **WHEN** a Gate pad starts before a seam, tempo changes and release arrives in the next loop
- **THEN** its saved duration equals the accepted musical interval across the seam
- **AND** later playback follows that duration without an unfinished or duplicated note

#### Scenario: End recording before the gate releases
- **WHEN** recording ends with a Gate pad held and its physical release arrives later
- **THEN** the saved gate ends at the accepted pass boundary
- **AND** the late release cannot change that row or a newer pass

#### Scenario: Retrigger and clamp a recorded gate
- **WHEN** one pad retriggers before release or remains held beyond 245760 ticks
- **THEN** the earlier gate closes at retrigger and a duration cannot exceed 245760 ticks
- **AND** existing manual-to-manual sound release rules remain unchanged

### Requirement: Recording feedback remains bounded

Recording SHALL NOT allocate, deallocate, lock, log, access files or wait in the audio callback. Feedback overflow SHALL count omitted onsets and warn while manual sound and existing playback continue. Lost release feedback SHALL NOT leave an unfinished gate. Recovery SHALL NOT fabricate or replay dropped onsets. End-of-pass state SHALL remain recoverable under pressure.

#### Scenario: Capture overflow preserves sound and complete recoverable rows
- **WHEN** capture feedback fills while pads play, release and end recording
- **THEN** existing sound continues without callback allocation or waiting and omitted onsets increment a warning count
- **AND** recoverable gates finish, the pass can complete, and recovery creates no late-hit burst

### Requirement: The raw hit limit applies during recording

A1 SHALL retain at most 4096 raw rows, including hidden rows and pending gate reservations. Further onsets SHALL remain audible but unrecorded with a warning. Existing rows SHALL stay intact. Erasure SHALL make space available again. Limit refusal SHALL NOT create a pass Undo step without another pattern change.

#### Scenario: Record at the raw hit limit
- **WHEN** a pattern with 4095 rows accepts another onset and then further manual hits
- **THEN** at most 4096 rows remain and later refused onsets still sound manually
- **AND** hidden rows count toward the limit, and successful erasure permits later capture

### Requirement: Change pattern settings without a step editor

Compact pattern controls SHALL expose bars, quantize and strength on every current screen. Opening without A1 SHALL create an empty default A1. Bars SHALL clamp to 1–64 while recording is Idle. Quantize SHALL use all existing grids and strength steps of 10. Grid and strength changes during Recording SHALL preserve raw timing. Count-in and Finishing SHALL refuse setting edits.

#### Scenario: Set loop length before recording
- **WHEN** the owner opens pattern controls without A1, changes bars and reaches both length limits
- **THEN** an empty A1 uses the chosen length, values clamp at 1 and 64, without a TOML edit
- **AND** recording phases refuse length changes without changing hidden rows

#### Scenario: Change the recording grid without changing the raw take
- **WHEN** the owner changes among every quantize grid and strength 0, 50 and 100 during recording
- **THEN** later eligible occurrences follow those settings and raw timing stays unchanged
- **AND** CountIn and Finishing refuse those edits with the actual phase as the reason

### Requirement: Silent erasure uses held pad targets

Erasure SHALL toggle only during Recording. New pad presses SHALL silently target their absolute pads instead of recording notes. A held target SHALL erase raw rows whose compiled onsets fall within its accepted half-open playhead interval. An interval SHALL NOT erase an onset accepted after it closed. Other pads, untraversed onsets and hidden rows SHALL remain. A matching onset SHALL be silent before UI confirmation of removal.

#### Scenario: Erase only one pad under the playhead
- **WHEN** erasure is on and one pad is held while the playhead crosses selected beats
- **THEN** only that pad's rows at crossed compiled onsets disappear and those onsets do not sound
- **AND** the held pad creates neither a manual sound nor a new recorded hit

#### Scenario: Erase a seam and all rows behind a collision
- **WHEN** a held erase target crosses the seam and matching rows share a compiled tick
- **THEN** all matching raw collision rows disappear on both traversed sides of the seam
- **AND** release excludes its boundary tick while hidden and other-pad rows stay intact

#### Scenario: Delayed UI cannot extend an erase hold
- **WHEN** a raw MIDI erase press and release occur while the UI is delayed across several loops
- **THEN** only the accepted audio-time interval erases rows
- **AND** later loops cannot revive already erased rows before the UI catches up

#### Scenario: A later accepted onset survives an earlier erase interval
- **WHEN** erasure closes and the audio engine accepts a new manual onset in the same output frame
- **THEN** the new raw row remains while earlier matching rows disappear
- **AND** saving and pass Undo retain the correct rows without changing ordinary manual sound

### Requirement: Erasure feedback remains bounded

Erasure SHALL retain accepted audio-time intervals and removed rows until UI confirmation. Storage exhaustion SHALL refuse a new target with a warning, without a sounding hit or an untracked interval. Reused recording capacity SHALL NOT change an earlier Gate or inherit its playback history. The audio callback SHALL NOT allocate, free, lock or wait.

#### Scenario: Erasure pressure cannot create an untracked interval
- **WHEN** interval storage fills before accepted erasure can reach the UI
- **THEN** further targets remain silent and count a refusal without erasing outside retained intervals
- **AND** completion and project changes retain their bounded recovery rules

#### Scenario: Reused recording capacity keeps earlier Gates separate
- **WHEN** erasure frees rows after 4096 onsets and another Gate records in the same pass
- **THEN** the new press uses available capacity without changing any earlier completed duration
- **AND** stale completion or consumption cannot affect the new press

### Requirement: Erasure retains safe release behavior

Entering erasure SHALL NOT convert earlier ordinary holds into erase targets. Leaving erasure SHALL clear its target holds and require a fresh sounding press. Without keyboard releases, each erase pad press SHALL target one loop and then clear itself. Recording end, Stop and audio loss SHALL clear all targets without synthesizing a note.

#### Scenario: Leave erasure without a stuck or unintended note
- **WHEN** an ordinary pad is held before erasure, another pad becomes an erase target, and erasure ends before release
- **THEN** the earlier hold keeps its original release and the erase hold cannot start sound
- **AND** a fresh pad press returns to normal recording

#### Scenario: Erase one loop without keyboard releases
- **WHEN** a terminal without releases sends one erase pad press
- **THEN** its target lasts exactly one loop of musical time and clears automatically
- **AND** the display names this fallback without changing normal one-shot pad behavior

### Requirement: One recording pass makes one Undo step

A pass SHALL extend from recording start to accepted recording end. Its recorded rows, erasures and pattern-setting changes SHALL make one isolated Undo step when they change the pattern. Undo SHALL restore the pre-pass pattern without restoring pads, FX or Chop edits. Empty passes SHALL add no step. Consecutive passes SHALL NOT merge by time.

#### Scenario: Undo an overdub without losing earlier passes
- **WHEN** two consecutive passes add different hits and the owner undoes the second
- **THEN** only the first pass remains, including its prior settings and raw order
- **AND** unrelated pad, FX and Chop changes remain as they were

#### Scenario: Undo erasure and grid changes as one pass
- **WHEN** one pass erases rows, adds hits and changes quantize or strength
- **THEN** one Undo restores its pre-pass raw rows and pattern settings
- **AND** a pass with no pattern difference makes no Undo step

### Requirement: Pattern Undo cannot cross a newer structural edit

The dedicated pattern Undo action SHALL work on every screen. During recording it SHALL first finish and undo the current pass, never an older edit when that pass is empty. While Idle it SHALL require a pass at the top of common Undo history. A newer common edit SHALL cause refusal with instructions, not an out-of-order restore. Existing local Chop Undo SHALL remain separate outside an active pass.

#### Scenario: Undo a pass from another screen without crossing a swap
- **WHEN** a completed pass is the latest common step and pattern Undo runs on Chop or Settings
- **THEN** the pass is undone without changing the screen or local Chop state
- **AND** a newer pad swap instead blocks pattern Undo until that newer step is undone normally

### Requirement: Stop and interrupted audio finish recording safely

Transport Stop SHALL close the current pass before resetting position. Audio loss SHALL retain recoverable captured rows and close known pending gates at the last accepted tick. It SHALL end erasure and report the interruption or dropped input. Missing callbacks SHALL NOT leave saving or exit permanently blocked. Restart SHALL not resume a previous recording phase.

#### Scenario: Stop closes the take before resetting the clock
- **WHEN** transport Stop occurs during recording with pending Gate notes
- **THEN** the pass uses the pre-reset accepted boundary and produces at most one Undo step
- **AND** transport resets and a later start cannot deliver old capture into the new pass

#### Scenario: Audio loss during finishing retains a saveable take
- **WHEN** audio stops before a requested finish can receive another callback
- **THEN** the recoverable take finishes at the last coherent accepted tick with a warning
- **AND** known gates close and saving or exit can continue without a fabricated audio acknowledgement

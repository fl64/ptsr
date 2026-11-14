# patterns Specification

## Purpose
Play a saved pad rhythm through the project beat clock. Keep saved timing, exact loop boundaries and manual playing independent of interface redraws.

## Requirements

### Requirement: One looped pattern

The system SHALL support zero patterns or one pattern named `A1`, with 1–64 bars of 4/4. A bar SHALL contain 3840 ticks. Transport start SHALL play from tick zero and repeat the pattern. Repeated explicit start SHALL NOT restart it. Without a pattern, transport and metronome SHALL retain their existing behavior.

#### Scenario: Pattern repeats from its first hit
- **WHEN** a two-bar A1 pattern runs for four loops at 120 BPM
- **THEN** every hit plays once per loop, including its tick-zero hit
- **AND** repeated explicit start does not restart the loop

#### Scenario: Empty pattern does not stop the beat clock
- **WHEN** transport runs with no pattern or a pattern with no hits
- **THEN** no sequenced pad sounds and the clock and enabled metronome continue

### Requirement: Raw pattern hits

Each raw hit SHALL contain an integer tick from 0–245759, absolute pad from 0–127, velocity from 1–127 and gate length from 0–245760 ticks. A pattern SHALL contain at most 4096 raw hits. Playback velocity SHALL equal velocity divided by 127. Changing the open bank SHALL NOT change an event's pad.

#### Scenario: Pattern addresses do not follow the open bank
- **WHEN** hits address pads 0 and 127 while the owner changes the visible bank
- **THEN** the same bank 1 and bank 8 sounds play with their saved velocities

### Requirement: Quantize rounding and collisions

At capture, the nearest grid midpoint SHALL choose the later line. Fractional placed ticks SHALL round to the nearest integer, with half choosing the later tick. Same-pad hits at the same saved tick SHALL play once. The higher velocity SHALL win, then the longer gate length, then the earlier saved row. Saved rows SHALL NOT merge during playback.

#### Scenario: Pattern midpoint and collision rules are deterministic
- **WHEN** captured hits lie at exact grid midpoints or saved hits share a pad and tick
- **THEN** rounding follows the later-line rule and each collision uses the specified winner
- **AND** different pads on that tick retain their separate hits

### Requirement: Hidden hits survive loop shortening

Playback SHALL exclude saved hits at or beyond the current loop length. These hits SHALL remain saved and return when the loop grows. At capture, a hit placed at the loop end SHALL wrap to tick zero before save. Shortening the loop SHALL NOT wrap hidden hits into the start.

#### Scenario: Pattern shortening cannot wrap hidden hits into the start
- **WHEN** a two-bar pattern shortens to one bar with hits before, on and after tick 3840
- **THEN** hits on and after tick 3840 stay saved but do not play or wrap into the start
- **AND** growing back restores them, while a hit captured at the seam plays at tick zero once

### Requirement: Pattern timing uses audio frames

Pattern onsets and scheduled releases SHALL occur within one output frame of their clock positions. Timing SHALL remain independent of redraws and audio block lengths. Loop seams and callback boundaries SHALL NOT miss or double events. Empty processing SHALL NOT consume events, deadlines or musical time.

#### Scenario: Pattern timing is independent of callback size
- **WHEN** identical patterns render at 40, 120, 172 and 300 BPM with 1-frame, 128-frame, 1024-frame and irregular blocks
- **THEN** each onset and release stays within one output frame of its clock position
- **AND** tick-zero hits and events near block or loop boundaries occur once

#### Scenario: Pattern empty blocks preserve its first event
- **WHEN** empty blocks occur before the first non-empty block or between playing blocks
- **THEN** only processed output frames advance the pattern and its release deadlines

### Requirement: Sequenced hits preserve pad sound rules

Sequenced hits SHALL use the current pad mode, volume, pitch, source bounds, envelope, filter, drive, sends and mute group. Same-pad retriggers and group cuts SHALL use their existing short fade. Simultaneous sequenced hits SHALL run in ascending absolute-pad order after block-start manual commands. Empty, missing and loading pads SHALL be silent without later replay.

#### Scenario: Pattern sound matches ordinary pad playing
- **WHEN** a pad renders through manual commands and equivalent pattern events with Pitch Gate Off, matching velocity, settings and gate releases
- **THEN** output differs by at most 0.000001 per sample
- **AND** mute groups cut the same sounds while unrelated groups and ungrouped pads ring on

#### Scenario: Pattern cuts do not start before the event
- **WHEN** a sequenced same-pad retrigger or mute-group hit falls inside an audio block
- **THEN** the previous voice continues unchanged until that event frame and starts its existing cut there

#### Scenario: Simultaneous pattern hits use a stable order
- **WHEN** a manual block-start hit and simultaneous pattern hits share a pad or mute group
- **THEN** manual commands retain accepted order and the pattern hits follow in ascending absolute-pad order
- **AND** the last applicable trigger determines the surviving voice under existing cut rules

#### Scenario: Unavailable pattern pads never replay missed hits
- **WHEN** an event addresses an empty, missing or loading pad and that pad becomes available later
- **THEN** the earlier hit stays silent and only future events can play it

### Requirement: Sequenced gate lengths

A gate-mode event SHALL release its own note after its saved length through the pad's release envelope. Zero-length gate events SHALL be silent. One-shot events SHALL ignore gate length. Gate deadlines SHALL retain their musical duration across loop seams and tempo changes. Tempo changes SHALL NOT reset the clock or repeat events.

#### Scenario: Pattern gate length crosses a loop seam safely
- **WHEN** a gate note near the loop end has a positive length extending into the next loop
- **THEN** only that note releases at its saved musical deadline through its release envelope
- **AND** a zero-length gate note is silent while a one-shot with zero length plays normally

#### Scenario: Pattern tempo changes preserve pending musical deadlines
- **WHEN** tempo changes while a pattern and gate notes run
- **THEN** future hits and releases follow the new tempo without resetting position or losing metronome phase

### Requirement: Automatic releases retain note ownership

A scheduled release SHALL affect only the note that created it. Retrigger, voice stealing, source replacement, pad swap, stop and project replacement SHALL NOT leave a deadline able to release another note. Manual release SHALL NOT release a newer sequenced note. Existing manual-to-manual release behavior SHALL remain unchanged.

#### Scenario: Old pattern release cannot cut a new manual hit
- **WHEN** a sequenced gate note or Pitch Gate note is followed by a manual hit on that pad before its old deadline
- **THEN** the old deadline cannot release or cut the manual voice

#### Scenario: Manual release cannot cut a newer pattern note
- **WHEN** a held manual pad is retriggered by the pattern before the manual control is released
- **THEN** the manual release leaves the pattern note to its own deadline

#### Scenario: Pattern deadlines cannot follow a reused voice slot
- **WHEN** notes retrigger, are stolen, change source or end before their deadlines and their slots are reused
- **THEN** no stale deadline releases any replacement voice

### Requirement: Pattern stop and restart

Stop SHALL prevent new pattern hits, release sequenced gate notes and cut active Pitch Gate notes through their respective release paths. Other one-shot tails, manual voices, previews and shared effects SHALL retain their existing behavior. Stop SHALL clear pending pattern deadlines and reset position. Restart SHALL start with one tick-zero event.

#### Scenario: Pattern stop preserves unrelated tails and clears deadlines
- **WHEN** Stop occurs with sequenced gates, Pitch Gate notes, ordinary one-shots, manual voices, previews and effect tails
- **THEN** only the sequenced gates and Pitch Gate notes receive automatic releases or cuts
- **AND** later restart has no old deadlines and plays its first event once

### Requirement: Same-project pattern replacement

Replacement SHALL take effect atomically at an audio block boundary, without resetting transport or metronome phase. It SHALL release old sequenced gate and Pitch Gate notes. Events whose nearest frame precedes that boundary SHALL NOT replay. Events at or after it SHALL follow the replacement. Removing a pattern SHALL leave transport running.

#### Scenario: Pattern replacement keeps phase without a duplicate hit
- **WHEN** a replacement adds events before, on and after the next block boundary while transport runs
- **THEN** past events do not replay and each applicable new event sounds once
- **AND** old gate deadlines cannot affect new notes and removing the pattern leaves the clock running

### Requirement: Bounded pattern playback under resource pressure

Playback SHALL NOT allocate, deallocate, lock, log, access files or wait in the audio callback. Retirement pressure SHALL defer whole replacements while the old pattern continues. If automatic voice retirement cannot proceed, the hit SHALL be omitted and counted, not delayed. The UI SHALL warn about dropped pattern hits. Normal dense playback SHALL drop no hit.

#### Scenario: Pattern retirement pressure does not block or free in the callback
- **WHEN** retirement is full during playback, replacement, obsolete-command disposal or project transition
- **THEN** references remain owned until safe retirement without callback allocation, deallocation or waiting
- **AND** deferred replacements stay whole, omitted automatic hits increment the warning count, and recovery produces no late-hit burst

### Requirement: Bounded automatic release storage

Automatic release storage SHALL be bounded. Stale entries SHALL expire without affecting voices. When capacity is exhausted, the earliest pending deadline SHALL release only its own note before a new entry is stored. Equal deadlines SHALL choose the oldest note. The callback SHALL NOT allocate storage or use a pad-wide release for this case.

#### Scenario: Pattern release capacity cannot release another owner
- **WHEN** the fixed release capacity is reached and another deadline needs storage
- **THEN** only the oldest applicable pending note releases and the new deadline remains tracked
- **AND** manual voices and reused note identities remain unaffected

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

When a pass completes, each fresh row SHALL replace older rows of the same absolute pad at the same saved tick. The most recently captured row SHALL stay, also between two fresh rows of one pass. Other ticks, other pads and rows hidden by the loop length SHALL stay. Replacement SHALL NOT occur while the pass records. Pass Undo SHALL restore replaced rows. Load SHALL NOT remove rows.

#### Scenario: A replayed step keeps only the newest hit
- **WHEN** a pass adds a hit on a pad whose older row has the same saved tick, and the pass ends
- **THEN** only the newest row remains at that pad and tick, with its velocity and Gate length
- **AND** later loops play that row once

#### Scenario: Two hits in one cell of one pass keep the last
- **WHEN** the owner hits one pad twice inside one 1/16 cell in one pass with Quantize 1/16
- **THEN** after the pass one row remains at that tick, from the second hit
- **AND** with Quantize 1/32 the two hits land in two cells and both stay

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

Compact pattern controls SHALL expose bars, quantize and strength on every current screen. Opening without A1 SHALL create an empty default A1. Bars SHALL clamp to 1–64 while recording is Idle. Quantize SHALL use all existing grids and strength steps of 10. Grid and strength changes SHALL apply to later captured rows only. Count-in and Finishing SHALL refuse setting edits.

#### Scenario: Set loop length before recording
- **WHEN** the owner opens pattern controls without A1, changes bars and reaches both length limits
- **THEN** an empty A1 uses the chosen length, values clamp at 1 and 64, without a TOML edit
- **AND** recording phases refuse length changes without changing hidden rows

#### Scenario: Change the recording grid without changing the raw take
- **WHEN** the owner changes among every quantize grid and strength 0, 50 and 100 during recording
- **THEN** later captured rows follow those settings and earlier rows keep their saved ticks
- **AND** CountIn and Finishing refuse those edits with the actual phase as the reason

### Requirement: Silent erasure uses held pad targets

Inside a pass, erasure SHALL toggle only during Recording. New pad presses SHALL silently target their absolute pads. A held target SHALL erase raw rows whose compiled onsets fall within its accepted half-open playhead interval. An interval SHALL NOT erase an onset accepted after it closed. Other pads, untraversed onsets and hidden rows SHALL remain. A matching onset SHALL be silent before UI confirmation of removal. Outside a pass, erasure SHALL follow "Erase whole pads outside recording".

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

### Requirement: Pattern view is independent of capture quantize

The view SHALL support 1/4, 1/8, 1/8T, 1/16, 1/16T, 1/32, 1/32T and 1/64, with 1/16 as the default. View steps SHALL follow this order of cell size, and each finer step SHALL have a smaller cell. Recording Quantize Off SHALL NOT remove view cells. View changes SHALL preserve saved rows, quantize, strength, velocities, Gate lengths, dirty state and Undo history. Viewing absent A1 SHALL NOT create it.

#### Scenario: Pattern view divisions preserve the take
- **WHEN** each view division shows patterns captured at every quantize grid and strengths 0, 50 and 100
- **THEN** saved rows and recording settings remain unchanged and Off still has a readable view
- **AND** changing the view or inspecting absent A1 creates no edit or history entry

### Requirement: Pattern cells retain source event identity

Each visible saved row SHALL occupy the half-open view cell containing its saved tick. Rows beyond the current loop length SHALL stay hidden. A seam-wrapped saved row SHALL appear at loop start. Off-grid positions and multiple saved rows SHALL remain distinguishable, including same-tick collisions. The view SHALL NOT merge saved rows.

#### Scenario: Pattern cells show off-grid events and collisions
- **WHEN** off-grid same-pad saved rows share a cell or tick
- **THEN** the cell reports their multiplicity and keeps each row in saved-tick and original-row order
- **AND** no saved row changes merely because the view groups it

#### Scenario: Pattern view preserves hidden rows and the loop seam
- **WHEN** a loop shortens with hidden rows and a visible row captured at the seam
- **THEN** only the visible seam row appears at loop start and hidden rows remain saved but absent from cells
- **AND** growing the loop restores the hidden rows at their corresponding positions

### Requirement: Pattern pages preserve musical position

A page SHALL contain the most whole bars that fit the strip width, up to the whole loop, starting at a bar line. If one bar does not fit, a page SHALL contain half a bar, then one beat. A page SHALL NOT split a beat. A running pattern SHALL show the page that holds the playback position. A stopped transport SHALL show the first page.

#### Scenario: Pattern pages cover straight and triplet bars
- **WHEN** the owner views every page of each view across one-bar, two-bar and 64-bar patterns at 80 and 120 columns
- **THEN** every cell appears once in order with complete beats, and a two-bar 1/16 loop fits one page at both widths
- **AND** the page follows the playback position, and a stopped transport shows the first page

### Requirement: Erase whole pads outside recording

Outside a recording pass, erasure SHALL be a mode of Pads. On other screens, the toggle SHALL change nothing. In the mode, a new pad press SHALL make no sound. It SHALL erase every raw A1 row of its absolute pad, including rows hidden by the loop length. Each erasure SHALL be one common-history Undo step. A press on a pad without rows SHALL change nothing. Releases SHALL do nothing. The pad's sound, other pads and settings SHALL stay unchanged.

#### Scenario: Erase one whole pad while stopped
- **WHEN** pads Q and W have A1 hits with a hidden Q row and transport does not play
- **AND** the owner turns on erasure on Pads and presses pad Q
- **THEN** pad Q makes no sound, every row of pad Q disappears, and the rows of pad W stay
- **AND** one Ctrl+Z restores the rows of pad Q, and a press on a pad without rows changes nothing

### Requirement: Whole-pad erasure needs hits

Whole-pad erasure SHALL start only when A1 has at least one raw row.

#### Scenario: Erasure needs hits in A1
- **WHEN** A1 is absent or has no rows and the owner turns on erasure outside a pass
- **THEN** erasure stays off and an Info notice says that A1 has no hits to erase

### Requirement: Whole-pad erasure during playback

Whole-pad erasure SHALL NOT stop or move transport. After the erasure, the pad SHALL make no new sequenced hit. A voice that sounds at the erasure SHALL finish its normal release.

#### Scenario: Erase one whole pad during playback
- **WHEN** A1 plays pads Q and W, and the owner turns on erasure and presses pad Q while a Q voice sounds
- **THEN** the Q voice finishes its release without a click, and later loops play only pad W
- **AND** the rendered output has no click at the erasure

### Requirement: Erasure modes do not change into each other

Erasure outside a pass SHALL end when a recording pass starts or the project changes. It SHALL also end when the screen changes from Pads, and on Esc or a second toggle. Erasure inside a pass SHALL end with the pass. A pass SHALL start with ordinary recording. Erasure SHALL NOT start during CountIn or Finishing. Ordinary holds from before erasure SHALL keep their original releases.

#### Scenario: Starting a pass leaves whole-pad erasure
- **WHEN** whole-pad erasure is on and the owner starts recording
- **THEN** erasure turns off before CountIn, and pad presses in the pass record hits
- **AND** the start erases no row

#### Scenario: A pass end does not leave whole-pad erasure on
- **WHEN** held erasure is on in a pass and the pass ends
- **THEN** erasure is off, and the next pad press sounds and erases no row

#### Scenario: Leaving Pads leaves whole-pad erasure
- **WHEN** whole-pad erasure is on, and the owner opens Chop and then returns to Pads
- **THEN** erasure is off, and pad presses on Chop and Pads sound as usual

### Requirement: Clear all of A1

`pattern.clear` SHALL ask before it clears every raw A1 row. Only the confirmation SHALL clear rows. A1 SHALL keep its bars, quantize and strength. Clearing SHALL be one common-history Undo step. The keep answer SHALL change nothing. During an active or pending recording pass, the command SHALL refuse and say to end the pass.

#### Scenario: Clear A1 asks first
- **WHEN** the owner runs Clear A1 and keeps, then runs it again and confirms
- **THEN** keep changes nothing, and confirm clears every row and keeps the settings
- **AND** one Ctrl+Z restores every row

#### Scenario: Clearing during a pass refuses
- **WHEN** a recording pass is active and the owner runs Clear A1
- **THEN** a notice says to end the pass first
- **AND** the rows stay unchanged and no history entry appears

### Requirement: Quantize applies when a hit is captured

A captured row SHALL get its tick from the Quantize and Strength of A1 at its capture. The 96 rounding rules SHALL apply. Off SHALL keep the captured tick. A Gate duration SHALL stay as captured. A later Quantize or Strength change SHALL NOT move a saved row.

#### Scenario: A saved hit keeps the quantize of its capture
- **WHEN** the owner records a hit with Quantize 1/4, then sets Quantize 1/16 and records another hit
- **THEN** the first row stays on its 1/4 line and the second row lands on the nearest 1/16 line
- **AND** playback plays both rows at their saved ticks

#### Scenario: Strength and Off set the captured tick
- **WHEN** the owner records one off-grid hit with Quantize 1/16 at strengths 0, 50 and 100, and with Off
- **THEN** the saved ticks move 0, 50 and 100 percent toward the line, and Off keeps the captured tick
- **AND** the Gate duration of each row stays as captured

### Requirement: Playback plays saved ticks

Playback SHALL play each visible row at its saved tick. It SHALL NOT quantize the rows again. Same-pad rows on one tick SHALL play once by the 96 collision order.

#### Scenario: A settings change does not move playback
- **WHEN** A1 plays saved rows and the owner changes Quantize and Strength through every value
- **THEN** the rendered onsets stay at the saved ticks within one output frame
- **AND** the rendered output has no clipping, non-finite sample or click

### Requirement: A1 overview shows every bank in one vertical list

The A1 overview SHALL keep all eight banks in its fixed index. By default, its vertical list SHALL show rows with visible hits and the selected pad in the current bank. It SHALL keep pad order within each bank. Each shown row SHALL show the pad key, pad name, and visible events at the selected view division. The view SHALL use compiled playback ticks without changing saved raw hits. It SHALL keep hidden raw hits hidden and show visible seam hits at the loop start.

#### Scenario: Overview shows occupied and selected pad rows
- **WHEN** A1 has hits on several pads and banks, including off-grid hits, two source hits in one cell, and a selected empty pad
- **THEN** the list shows only pads with visible hits and the selected pad in bank order
- **AND** each occupied cell shows its event type and count without merging source hits or changing A1

#### Scenario: Overview shows all pad rows and source events
- **WHEN** the owner uses Alt+H with hits on several pads and banks
- **THEN** each shown bank keeps all 16 pad rows in order, including empty rows
- **AND** occupied cells show their event type and count without changing A1

#### Scenario: Overview filters hidden hits before seam wrapping
- **WHEN** A1 has a hidden row beyond its shortened length and a visible row that quantizes to the seam
- **THEN** the hidden row does not appear in any overview cell or active-row count
- **AND** the visible seam row appears once at the loop start without changing the raw rows

### Requirement: Overview follows the shared view division and bank

The overview SHALL use the same runtime view division as the Pads strip. Existing bank controls SHALL move the viewport to the selected bank section and keep its selected row visible. Changing the division or current bank SHALL change only the overview projection and selection. It SHALL NOT change playback, quantize, raw hits, history, or dirty state. The view division SHALL return to 1/16 after a project change, as on Pads.

#### Scenario: Overview bank and view changes leave A1 unchanged
- **WHEN** the owner switches banks and view divisions while A1 plays, including a bank with no visible hits
- **THEN** the selected bank and pad appear in the viewport with the selected division, and A1 keeps its timing
- **AND** raw rows, quantize, history, and dirty state stay unchanged

### Requirement: Overview can hide empty bank sections

The overview SHALL show all eight bank numbers in a fixed index and mark banks with visible A1 hits. By default, it SHALL hide empty pad rows and bank sections except the selected pad and current bank. The saved `pattern.overview_empty_toggle` action (`Alt+H` by default) SHALL show all 16 rows in all eight banks. Use it again to restore the filtered view. A bank with only hidden raw hits SHALL count as empty. This runtime toggle SHALL NOT change A1 or pad selection.

#### Scenario: Overview hides empty banks without losing access
- **WHEN** A1 has visible hits in two banks and hidden raw hits in another bank
- **THEN** the fixed index shows all eight banks while the list starts with occupied and selected rows only
- **AND** Alt+H shows all sections and rows, then restores the filtered view without changing A1

#### Scenario: Overview bank navigation reaches a hidden empty bank
- **WHEN** the owner selects a bank with no visible A1 hits while empty rows are hidden
- **THEN** that bank section appears with the selected pad row and the bank index marks it as current
- **AND** another bank selection restores the ordinary filtered view

### Requirement: A1 step correction needs an explicit mode

The owner SHALL enter correction only from the A1 overview with an existing A1. Entry and exit SHALL leave transport, raw hits, dirty state and history unchanged. A conflicting input owner or pending capture SHALL refuse entry without ending its work.

#### Scenario: Step correction entry keeps A1 playing
- **WHEN** the owner requests correction with absent, empty, stopped and playing A1
- **THEN** absent A1 gives the effective record action, while existing A1 enters without moving transport
- **AND** first entry selects the valid playback cell or the first cell, and reentry retains the same-project cursor

#### Scenario: Step correction refuses conflicting work
- **WHEN** capture, recovery, placement, marking, pad edit, pattern controls, audio recording or a question owns input
- **THEN** correction refuses with a next action and does not change the existing work
- **AND** text, pickers, learning and original pad releases keep their priority

### Requirement: A1 step correction removes one identified hit

An edit pad press SHALL target only its pad in the selected time cell. One visible event SHALL be removable on the first press. A cell with several events SHALL require an explicit event choice before removal. Repeats and releases SHALL remove nothing and make no immediate sound. An empty cell SHALL follow the addition rule.

#### Scenario: Step correction removes one hit without pad sound
- **WHEN** the owner presses a pad with one visible event at the selected cell
- **THEN** exactly that raw row disappears while other hits, pad settings, and held tails stay unchanged
- **AND** the press, repeats, and release make no new sound or removal

#### Scenario: Step correction chooses one of several hits
- **WHEN** the owner first presses another pad with several events in the cell, selects an event, and presses the pad again
- **THEN** the first press selects without removal and the next press removes only the chosen raw row
- **AND** other rows, including same-tick collisions, keep their timing, velocity, and Gate lengths

#### Scenario: Step correction rejects a stale event identity
- **WHEN** a project or pattern change invalidates a selected event before removal
- **THEN** the old choice cannot remove a replacement row with a reused index
- **AND** selection refreshes without an edit or sound in the new project

### Requirement: A1 step cursor uses the shared view division

The edit cursor SHALL keep its own time cell and page while playback continues. Bank changes SHALL keep time and select the new bank. A view change SHALL select the new cell that contains the old cell's first tick. Cursor and view changes SHALL NOT edit A1.

#### Scenario: Step cursor stays independent of playback
- **WHEN** A1 crosses beats, pages and the loop seam while the owner moves the cursor or changes banks and view division
- **THEN** the cursor stays on its chosen page and the shared view remains aligned to complete beats
- **AND** A1 timing, raw rows, history and dirty state stay unchanged

### Requirement: A1 step correction keeps ordered audio publication

The App SHALL check a replacement before mutation and publish each accepted addition or removal at an audio boundary. It SHALL keep transport and active note deadlines. Queue pressure SHALL retain accepted edits in order without callback allocation, blocking, or replay of past onsets.

#### Scenario: Step removal survives publication pressure
- **WHEN** removal and another ordered edit meet command or retirement pressure while A1 and Gate notes play
- **THEN** accepted edits remain whole and ordered, past hits do not replay, and started notes keep their tails
- **AND** the audio callback allocates, destroys, and blocks on no new work

#### Scenario: Step addition survives publication pressure
- **WHEN** addition and another ordered edit meet command or retirement pressure while A1 and Gate notes play
- **THEN** accepted edits remain whole and ordered, past hits do not replay, and started notes keep their tails
- **AND** the audio callback allocates, destroys, and blocks on no new work

### Requirement: A1 step removal has one-event Undo and safe save

Each successful removal SHALL create one common-history Undo step. Pattern Undo SHALL respect newer unrelated edits. Save SHALL write corrected raw rows in format 5 without editor state. A save refusal SHALL keep the correction and Undo.

#### Scenario: Step removal Undo restores one hit
- **WHEN** the owner removes two hits, undoes the latest and makes a newer pad or FX edit
- **THEN** each permitted Undo restores only its identified hit and does not cross the newer edit
- **AND** cursor movement and empty presses create no history or redo

#### Scenario: Step correction saves without editor state
- **WHEN** a correction meets save refusal, then saves and the owner reopens the project
- **THEN** refusal keeps unsaved work and Undo, and the saved raw rows contain only the intended correction
- **AND** transport starts stopped, edit mode closes and view division returns to 1/16

### Requirement: Overview step cells keep fixed width

Each overview step SHALL use three terminal cells at 80x26 and 120x40. A one-cell separator SHALL mark each beat. Straight and triplet views SHALL use whole beat groups. Step numbers SHALL align with their event cells. The system SHALL leave unused width blank rather than stretch cells. It SHALL retain beat shading, bar borders, the edit cursor, and the live playback plate.

#### Scenario: Overview fixed cells align every division
- **WHEN** the owner views each straight and triplet division at 80x26 and 120x40
- **THEN** each step number stays over a three-cell step and each beat starts after one separator
- **AND** bar, beat, cursor, and moving playback marks remain visible without changing A1

### Requirement: Overview shows three velocity levels

An occupied cell SHALL use low, medium, or high brightness for its highest visible velocity. The levels SHALL be 1–42, 43–84, and 85–127. A single on-grid hit SHALL retain `■`. A single off-grid hit SHALL retain `~`. Several hits SHALL retain their count mark. The step editor SHALL show the selected event's numeric velocity. Brightness SHALL NOT be the only mark that identifies a hit.

#### Scenario: Overview velocity levels keep event marks
- **WHEN** visible hits have velocities 1, 42, 43, 84, 85, and 127, including off-grid and multiple-hit cells
- **THEN** the three brightness levels follow those boundaries and the glyph still shows hit type or count
- **AND** the selected event shows its numeric velocity without changing raw rows

### Requirement: A1 step edit adds one hit at an empty cell

In step edit, a new keyboard or raw MIDI pad press on an empty cell SHALL add one raw A1 hit for that absolute pad. The hit SHALL use the first tick of the cursor cell and keep the received velocity. Shift plus a keyboard pad SHALL give full velocity. A OneShot pad SHALL store zero Gate ticks. A Gate pad SHALL store one current view-cell length. The edit press, repeat, and release SHALL make no immediate pad sound. Only the first press SHALL edit.

#### Scenario: Step input adds at an empty cursor without sound
- **WHEN** the owner presses a keyboard pad normally and with Shift, then a raw MIDI pad at an empty cursor cell
- **THEN** each accepted press adds one hit at that cell with its source velocity and no immediate sound
- **AND** repeats and releases add nothing, while ordinary pad play works after leaving edit

#### Scenario: Step input keeps Gate length and saved tick
- **WHEN** the owner adds OneShot and Gate pads at a fine view cell while capture quantize uses a coarse grid
- **THEN** both hits play at the selected cell, the OneShot has zero Gate ticks, and the Gate lasts one view cell
- **AND** changing view or quantize later does not move either saved hit

### Requirement: Step edit cursor marks one time column

The edit cursor SHALL mark one time column across the step header and every visible pad row.
It SHALL NOT frame one pad cell as the only edit target. J/L and learned MIDI
`edit.value_dec` and `edit.value_inc` SHALL move this column one step at a time. The time
cursor SHALL remain distinct from the live playhead.

#### Scenario: Step cursor shows one time across pad rows
- **WHEN** the owner moves the step with keyboard or learned MIDI controls at 80x26 and 120x40
- **THEN** the header and every visible pad row mark the same step without a single-cell frame
- **AND** the moving playback mark stays distinct

### Requirement: Step pad press shows its row target

A pad press SHALL select that pad's row and keep it visible before the system changes its
hit at the selected step. A press on another pad SHALL leave hits on other pad rows
unchanged. The selected row SHALL use a mark separate from the time cursor.

#### Scenario: Step pad press changes only its own row
- **WHEN** the owner selects a step on pad 1 and presses Q in the same bank
- **THEN** the time column stays on that step and Q becomes the selected row
- **AND** only Q gains or loses a hit while pad 1 keeps its hit

### Requirement: A1 step addition refuses unsafe work

The system SHALL check the full replacement before changing A1. It SHALL refuse at 4096 raw rows or when a conflicting input owner blocks editing. Refusal SHALL leave A1, Undo, dirty state, and sound unchanged. A new saved tick SHALL play at its stored position even when capture quantize differs from the view division.

#### Scenario: Step input refuses a full or stale edit
- **WHEN** A1 has 4096 rows, or a project or pattern change makes an edit target stale
- **THEN** the refused press adds no row and leaves history, dirty state, and sound unchanged
- **AND** the owner sees a next action without losing existing work

### Requirement: A1 step addition has one-event Undo and safe save

Each accepted addition SHALL create one common-history Undo step. Undo SHALL remove only that added row and SHALL not cross a newer unrelated edit. Save SHALL write the new raw row in format 5 without editor state. A save refusal SHALL keep the addition and Undo. Reopening SHALL restore the saved row with transport stopped and edit mode closed.

#### Scenario: Step addition Undo restores the prior A1
- **WHEN** the owner adds two hits, undoes the latest, and makes a newer pad or FX edit
- **THEN** each permitted Undo removes only its added row and does not cross the newer edit
- **AND** cursor movement, focus, repeats, and refused presses create no history

#### Scenario: Step addition saves without editor state
- **WHEN** an addition meets save refusal, then saves and the owner reopens the project
- **THEN** refusal keeps unsaved work and Undo, and the saved A1 contains the intended hit
- **AND** transport starts stopped, edit mode closes, and the view returns to 1/16

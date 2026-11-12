# patterns Delta

## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Silent erasure uses held pad targets

Inside a pass, erasure SHALL toggle only during Recording. New pad presses SHALL silently target their absolute pads instead of recording notes. A held target SHALL erase raw rows whose compiled onsets fall within its accepted half-open playhead interval. An interval SHALL NOT erase an onset accepted after it closed. Other pads, untraversed onsets and hidden rows SHALL remain. A matching onset SHALL be silent before UI confirmation of removal. Outside a pass, erasure SHALL follow "Erase whole pads outside recording".

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

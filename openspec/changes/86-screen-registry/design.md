## Context

See `proposal.md` for the reason and owner checks. The implementation baseline is `a1e4829` on `86-screen-registry`, after accepted change 85.

The plan sketches predate 85. The current code already packs screen, bank and independent mode flags into one atomic word.
Do not restore the plan's older exclusive-mode model or five separate atomics.

Relevant baseline locations:

| Location | Current responsibility |
| --- | --- |
| `crates/input/src/lib.rs`, `Screen`, `Focus::pack`, `Focus::unpack` | Screen variants and a separate numeric decoding table |
| `crates/input/src/lib.rs`, `Group`, `Keymap::apply_toml` | Group keys, titles, scopes and another screen-name table |
| `crates/input/src/midi.rs`, `Profile::from_toml`, `resolve_in`, `fx_binding` | Profile section names and special FX resolution |
| `crates/app/src/main.rs`, `goto`, `layer_group`, `screen_group` | Screen groups and the group to open in F12 |
| `crates/app/src/main.rs`, `help_groups`, `full_help`, `keymap_shadow_warnings` | Help rules and screen names in keymap diagnostics |
| `crates/app/src/dispatch.rs`, `App::apply`, `App::focus` | Screen-learn availability and input state publication |
| `crates/app/src/midi.rs`, `Shared::focus`, `Shared::input`, `dispatch` | State reads, profile resolution, original hold tracking and ordered UI delivery |

`Shared::focus` currently uses `Focus::unpack(...).unwrap_or_default()`.
`Shared::publish` accepts only valid packed state, so ordinary publication cannot produce an unknown screen.
The fallback remains unsafe if dispatch receives invalid bits. The new fault tests must exercise that real reader and dispatcher.

## Goals / Non-Goals

**Goals:**

- Store screen identity and shared rules once in `input`, without allocation or runtime registration.
- Preserve valid keyboard, MIDI, learning and help behavior. Make invalid-state rejection a separate, tested correction.
- Keep the atomic ordering, pending-input sequence and release identities from 85.

**Non-Goals:**

Do not make screen drawing or action execution generic. Their screen-specific match arms remain appropriate.
Do not add screens, modes, configuration migration, dependencies or audio callback work.
Do not treat missing optional owner bindings as an incomplete registry.

## Decisions

### 1. Use a small static registry

Add `Screen::ALL`, `code`, `from_code`, `key`, `title` and `rules` in `crates/input/src/lib.rs`.
Add a reverse section lookup for both keyboard and MIDI parsing.
Keep all identity data in one exhaustive definition. The methods read that definition rather than maintaining independent tables.

Keep the existing numeric encoding:

| Screen | Code | Section key | Screen group | UI edit group | F10 screen learn |
| --- | --- | --- | --- | --- | --- |
| Pads | 0 | `pads` | PADS | PADS · EDIT | Yes |
| Settings | 1 | `settings` | SETTINGS | None | No |
| Files | 2 | `files` | FILES | None | Yes |
| Chop | 3 | `chop` | CHOP | CHOP · EDIT | Yes |
| FX | 4 | `fx` | FX | None | Yes |

`Focus::pack` uses `Screen::code`. `Focus::unpack` uses `Screen::from_code`.
Codes 5–7 remain invalid. Banks, mode flags and reserved bits retain their layout and checks.
Do not use unchecked casts or transmute to decode a screen.

Preserve the `Group` API and its order:
`Global`, `Pads`, `PadsEdit`, `Settings`, `Files`, `Chop`, `ChopEdit`, `Fx`.
Screen-backed group keys and titles use registry metadata. Global and edit groups retain their existing keys and titles.

Enumeration order is not display order. Keep current help ordering and keymap-warning ordering, even where they differ from code order.
Registry tests check unique codes and keys, reverse lookup, group scopes and all five screens.
Application tests also check help and learning coverage. Do not test only that a title is nonempty.

Alternative: generate every enum and consumer with a new macro framework. Reject this because five screens need no new framework.
A dynamic plugin registry also adds unnecessary allocation and failure paths.

### 2. Separate identity from context-dependent rules

`ScreenRules` describes only the current shared distinctions:

- Availability of a UI edit group and F10 screen learn.
- Whether MIDI resolution accepts a separate edit layer.
- Whether MIDI resolution inherits the existing FX page controls.

The UI edit groups exist only for Pads and Chop. FX inherits page controls but has no separate MIDI edit layer.
Other MIDI screens retain their current handling of the supplied edit flag, including synthetic test states.
Do not mask the keyboard resolver's edit argument as a side effect of adding these rules.

Keep `fx_binding` and its existing priority logic. Replace screen-identity checks that select this policy with registry rules.
Do not turn inherited bindings into new stored copies. Keep explicit empty FX groups, seeding, migration and global clearing behavior.
Screen-specific FX action handling stays in App.

Use group metadata in `goto`, help construction and learning setup where it replaces a duplicate identity table.
Keep action lists, Settings tab help, Chop bounds help and rendering in their existing owners.
The separate `tui::Screen` enum also represents ProjectPicker. Do not merge it with input screens or register the picker as a new screen.

Baseline exception: `App::layer_group` and `screen_group` return Files for Settings.
The test `recorded_f4_from_each_screen_preserves_selection_scope_and_overlays` checks that result.
Do not silently change it to SETTINGS during this refactor. Preserve it as an explicit compatibility case in the relevant App helpers.
Registry identity for Settings remains SETTINGS. F10 remains unavailable there. The F12 learning list keeps its current path.

Alternative: replace every screen match with registry data. Reject this because screen-specific behavior is not duplicate identity metadata.
Correcting the Settings helper result is also outside this change.

### 3. Reject invalid shared state without blocking releases

Make the shared reader return checked state. Remove the default-Pads substitution from production reads.
Keep the acknowledged-sequence load before the state load, as required by 85.
Reject unknown screen codes, invalid banks and reserved bits through the same checked-state path.
Do not retain an earlier valid state as a substitute.

For each raw message, preserve this separation:

1. Process release identity from the existing device, channel and control tracking.
2. If state is invalid, reject any remaining new action before profile resolution or bank arithmetic.
3. Otherwise resolve and route the message with the valid snapshot and existing pending-input rules.

Disconnect handling and pending audio-release retries remain independent of screen decoding.
A tracked direct release still enters the existing audio release queue.
A tracked UI release still identifies its original hold for App. An untracked release cannot invent a target from invalid state.
Keep project epochs, queue rejection behavior and deferred hold identity unchanged.

An ignored message adds no numbered UI work. It cannot advance the acknowledgement of earlier work.
After valid publication, later messages use the usual sequence comparison. Do not flush earlier accepted work or replay ignored messages.

Emit one `log::error!` diagnostic per continuous invalid period from MIDI dispatch, never from the audio callback.
Include the invalid packed value. Reset the diagnostic guard when dispatch observes valid state.
Raw input logging may identify a rejected message, but it must not create a resolved action or claim a fallback screen.

Use test-only fault injection to place invalid bits into the real shared word.
Do not replace production routing with a test-only dispatcher. Cover codes 5–7, an invalid bank and reserved bits.

Alternative: keep default Pads or retain the last valid snapshot. Reject both because neither identifies the intended screen and bank.
Dropping all messages is also unsafe because it loses original gate releases.

## Tests

Each delta scenario gets a test with the scenario's name, or its snake-case form.
The main-spec coverage checker does not inspect active deltas. Check the new scenario-to-test mapping before implementation completes and before archive.

| Scenario | Test boundary |
| --- | --- |
| Existing screen bindings survive the registry change | Recorded keyboard and raw MIDI through real profiles, learned maps, dispatch and App |
| FX controls keep their priority through the registry | Paired command tests plus MIDI inheritance, explicit-empty, global-clear and persistence cases |
| Unknown MIDI screen rejects new input | Fault injection into shared state, raw notes, learned buttons and parameter CCs through real dispatch |
| Other invalid MIDI state has no fallback | The same production path with invalid bank and reserved bits |
| Invalid MIDI state preserves original releases | Direct and UI holds, release variants, disconnect, full audio queue and stale project events |
| Valid MIDI state resumes without rejected input | Invalid-to-valid sequence, rejected messages, earlier pending work and another invalid period |
| Each registered screen keeps its effective help | Paired screen actions, bottom help, F1, overrides, modifier and width cases |
| Screen learning keeps its existing entry points | Paired learn actions, edit groups, cancellation and Settings refusal |
| MIDI list keeps screen groups and their order | Paired Settings-entry actions and unchanged group names, order and scopes |

Shared behavior runs with keyboard events and raw MIDI through the real profile and dispatcher.
Packed-state corruption is MIDI-only. There is no keyboard event that corrupts this shared word.
Existing tests from 84 and 85 must retain their behavior assertions.
Representation-dependent assertions can use the new checked return type, but cannot weaken their ordering or state checks.

No audio algorithm changes. Reuse the three 85 render checks and listening references from the proposal.
Run `task check`, `task build` and strict OpenSpec validation before reporting implementation complete.
A successful benchmark does not close earlier callback timing issues.

## Risks / Trade-offs

- Wrong code or key association → Preserve the table above and test exact values, uniqueness and reverse lookup.
- Changed binding priority → Keep current assertions and test owner overrides, edit scopes, FX inheritance and explicit empty groups.
- Lost gate release during rejection → Track releases independently, including disconnect and audio queue retries.
- Broken input ordering → Preserve atomic source order and test recovery with earlier accepted work still pending.
- Misleading registry completeness → Check built-in sections and real help/learning consumers, not optional owner sections.

## Migration Plan

No data migration or profile reset is necessary. Keep existing profile files and configuration formats unchanged.
Keep each implementation step atomic and signed off, with `task check` green before its commit.
The invalid-state correction gets its own commit, separate from metadata consolidation.

Keep these artifacts in their own commit after approval and the required checks.
Implementation needs a new owner request after artifact review. Push and MR publication need separate authorization.
Update general status documents when the owner accepts the implementation plan. They are outside this artifact-only step.
After owner acceptance, synchronize and archive through the OpenSpec CLI. Do not archive by moving folders manually.
If rollback is necessary, revert only the affected 86 commits. Do not reset shared history or undo accepted closure of 85.

## Open questions

None that block this proposal. Physical-controller availability and listening results remain owner checks, not assumed facts.

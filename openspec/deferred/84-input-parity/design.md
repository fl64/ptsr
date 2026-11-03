## Context

This is the first implementation draft from `docs/plans/REFACTOR-INPUT.md`. It is kept
in `openspec/deferred/` during document preparation. Change 82 is accepted and archived.
Keymap cleanup 83a now comes first (`docs/plans/KEYMAP-CLEANUP.md`). Activate 84 on its own
branch after the preparation and 83a are merged; test the keys that 83a actually ships.
The J/L choice belongs to 83a, not to this draft.

The baseline for this draft is `origin/master` at `861424e`, the merge of PR #3.
Source locations below describe that baseline and can move. Recheck them after 83a:

- `crates/app/src/main.rs:1535`: keyboard Chop pad presses call `chop_pad`.
- `crates/app/src/main.rs:2722`: `chop_pad` handles unassigned slice pages, linked pad
  sounds and selection, and targets in an assign plan.
- `crates/app/src/main.rs:4438`: the MIDI Chop branch calls `play_slice` instead.
- `crates/app/src/midi.rs:35`: `Event::PadHit` carries the absolute pad and audio queue
  acceptance, but no hit velocity.
- `crates/app/src/midi.rs:286`: `dispatch` already resolves profiles and routes events.
  Its test helper `dispatched` at line 928 drives that path, but does not drive App.

Do not infer hardware support from the tests. The SMC-PAD profile is still a template.
The X-Session Pro has no pads; use its shipped profile for regression and explicit
learned pad bindings, not as an invented physical pad device.

## Goals / Non-Goals

Prove equivalent pad workflows through the two real input paths and fix the confirmed
Chop difference. Keep the keyboard behavior, MIDI velocity, releases, project epochs,
queue acceptance and X-Session safe defaults. Do not implement the later input model,
controller profile or sequencer here.

The broad parity goal covers all screens and modes in the roadmap. This delta specifies
Chop only, so this small fix does not silently promise fixes for other differences.
A newly found difference is a scope decision, not permission to change more behavior.

## Decisions

### One test driver, two input sources

Build a test-only `Drive` around the existing App test fixture. It supplies key press and
release events, raw MIDI bytes with a port identity, and a deterministic `frame` boundary.
Use existing dispatch helpers where possible. If extraction is needed, move only the
routing body into a shared production function in a separate behavior-neutral commit.
Do not replace real resolution with a test-only interpretation of the profile.

The MIDI route is byte parse -> actual profile resolver and seeding -> production MIDI
dispatch -> stamped UI events -> App handling. Capture the direct engine queue as well
as App commands. Drain and publish state at explicit boundaries, without timing sleeps.
Keep source channel, original held-pad identity and project epoch checks.

Use the shipped X-Session TOML and a small test pad profile (notes 36–51 and buttons on
CC25–29). The test profile is not a new shipped SMC-PAD map. The keyboard and MIDI passes
use matching velocities (100 or 127) for equality tests. Test lower MIDI velocities
separately; the keyboard is not required to produce every MIDI velocity.

Compare commands in order and the visible state: selected pad and slice, bank, plan,
in-hand object, marks and notices. Compare meaning, not input-debug lines that name the
source device. Assert that no extra direct pad hit accompanies a UI preview or assignment.

### Small Chop routing fix

Carry the original velocity in `Event::PadHit` and preserve it at both dispatch send sites.
Call `chop_pad` from the MIDI Chop branch with the pad's bank-local position and velocity.
Check the published bank and the existing same-frame behavior; do not add the future
acknowledgement protocol in 84. A new race finding goes into the divergences below.

Chop remains on the UI route. Normal performance pads still go directly to the audio
queue; the UI must not trigger them again. Keep the `accepted` field and epoch filtering.
Keep question and learn guards before the Chop handler. Assignment previews follow the
existing keyboard logic; do not add a new velocity rule for bare slice previews.

### Tests before completion

Run the shared workflows on Pads, pad edit, marking, file in hand, pad swap, pad copy
(from 83a), question, learn, Chop before assignment, linked-slice listen and by-hand assignment. Keep existing
X-Session tests unchanged. Add gate release coverage after a screen and bank change.
A difference outside the Chop delta blocks the affected step until it is documented and
its scope is agreed. Do not skip the test or change an assertion to accept an unknown
result.

Render paired linked-pad hits and low/high MIDI velocity through the implemented route
to `renders/84-input-parity.wav`. Check finite samples, no clipping, expected silence,
relative levels and the existing nearest-block timing. Do not claim input timestamps or
new latency guarantees. New scenario tests use the exact scenario names in the delta.
No scenario goes into the main specs until its test passes.

## Found divergences

- Confirmed at the baseline: MIDI Chop pads bypass `chop_pad`. This loses the keyboard
  slice selection and assign-target behavior and uses only the local slice number.
  Fix in 84; preserve original velocity for the linked-pad path.
- Other differences are not yet audited by the paired driver. In particular, the
  five-atomic snapshot and the pending-command race are work for 85, not fixes made by
  this preparation. Record observed results here during implementation.

## Risks / Trade-offs

- A helper that feeds resolved actions directly can pass while production MIDI fails.
  Test raw bytes through dispatch and App, including profile initialization.
- Adding velocity to an internal event affects senders, App matches and test fixtures.
  Find every `Event::PadHit` use; keep queue-acceptance behavior and release tracking.
- App handles Chop on the UI thread today. Preserve that route; do not move ordinary
  performance pad hits onto it or promise lower Chop latency.

## Open questions

- Does the paired driver expose another input priority difference? Default: record it,
  stop the affected task and propose a small delta or a separate change before fixing it.
- Which physical pad controller and preset will the owner use for acceptance? Default:
  the existing notes 36–51 fallback or an explicit learned pad binding. No SMC-PAD
  relative mode or port assumption is needed for 84.

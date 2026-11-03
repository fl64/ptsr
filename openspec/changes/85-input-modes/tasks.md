## 0. Scope and review

- [x] 0.1 Review the proposal, delta and design with the owner before code. Keep the
  narrow branch/scope decision, source-boundary exceptions and explicit ordering fix.
  Validate strictly, run full `task check`, and make an artifact-only signed-off commit
  when git authorization is given. Do not treat planning status as implementation approval.

## 1. Characterize and model existing input

- [x] 1.1 Add production-route characterization tests for existing text, project/folder
  pickers, F10 versus F12 learn, edit under questions, Chop with a hand state, overlapping
  hands and edit with marking. Keep source-specific expectations explicit, not invented
  parity. Extend Drive only with deterministic publish/drain fences where needed. Keep
  all 84 scenarios unchanged. Stop on an unexpected contract conflict. Run full
  `task check`; commit this test-only step with signoff.
- [x] 1.2 Add `Focus`, overlapping `Mode` flags, checked pack/unpack, `route_pad` and
  `App::focus()` without changing the active routes. Test the represented combinations,
  all screens/banks, invalid encodings, default state and every routing context. Keep
  edit independent and retain existing pre-routing guards. Run full `task check`;
  commit this model step with signoff.

## 2. Routing refactor and separate Chop repairs

- [x] 2.1 Replace the five MIDI Shared atomics with one packed Focus publication and
  one snapshot read per raw message. Use the snapshot for screen, edit, bank and the
  shared pad-route decision. Publish before dispatcher start. Mechanically adapt
  representation-bound tests without weakening their assertions. Add coherent-snapshot
  checks and keep direct/UI queues, original releases and input logging unchanged.
  Do not add acknowledgements yet. Run full `task check`; commit with signoff.
- [x] 2.1a Reproduce and repair mixed keyboard/MIDI release-target sharing on Chop,
  as separately approved by the owner. Add named tests for the three mixed-hold
  scenarios using keyboard events, raw notes and learned CC80 through production
  dispatch. Check both press and release orders, a screen change, a question, F10,
  and MIDI rejection by the actual full audio queue. Only keyboard hits may record
  the keyboard release target; preserve all other hit effects, original MIDI holds
  and queue acceptance. Use real offline audio and a timestamped waveform oracle;
  check finite/no-clip, silence, onset and independent gate release timing. Write
  `renders/85-mixed-holds.wav`. Keep the first failing assertions. Run full
  `task check`; commit this repair separately before 2.2. No sequencing or shared
  handler changes belong in this step.
- [x] 2.1b Reproduce and repair late direct notifications after Chop opens, as
  separately approved by the owner. Add named accepted/rejected regressions through
  raw notes and learned CC80, low/high velocity, linked listen, bare-slice listen,
  assignment and a bank change. Reject through the actual full MIDI audio queue.
  Carry explicit UI-unattempted/direct-accepted/direct-rejected sound disposition;
  only actual UI hits may start a Chop action. Keep original direct releases,
  epochs, existing guards, other hand priorities and Chop-entry hush. Mechanically
  adapt old event assertions without weakening them. Compare real offline audio
  with a timestamped oracle; check finite/no-clip, silence, onset and hush timing.
  Write `renders/85-late-midi.wav`. Keep red assertions, run full `task check` and
  commit this repair separately before 2.2. No sequencing or common handler yet.
- [x] 2.2 Make keyboard and MIDI call one `App::pad_hit`, with explicit input source and
  unattempted/direct-accepted/direct-rejected sound disposition. Keep wrapper undo,
  synchronization, existing source-specific priorities and original keyboard/MIDI hold
  bookkeeping. Use the real offline tests to prove no duplicate sound and no UI retry
  of a direct rejected hit. Keep velocity and all 84 regressions. Run full `task check`;
  commit this refactor with signoff, separately from the race repair.

## 3. Explicit controller ordering repair

- [x] 3.1 Add named regressions for all ten delta scenarios and first capture the
  failing same-update command/hit result through raw MIDI, real profiles and both
  actual offline audio queues. Then implement the dispatcher sequence and ordered
  Focus/ack publication for actions and UI pad hits, including no-ops, stale-event
  retirement, cross-device order and counter wrap. Keep normal direct hits and original
  direct releases independent of UI progress. Include callback progress past an older
  direct release before a deferred retrigger reaches App; keep the newer accepted UI
  hold's own release without changing non-deferred Chop merging. Extend UI candidate
  holds to deferred actual pads: preserve the accepted absolute pad, early note-off/CC0/disconnect,
  route changes, both direct/UI retrigger orders, full-queue rejection and pending
  releases, and project epochs. Pair shared logical outcomes with keyboard events;
  use MIDI-only checks for deferred transport and identity. Keep the original red
  assertions; commit only the final green test-and-repair state. The offline render
  tests must check one hit, finite/no-clip output, silence and release timing. Run
  full `task check`; commit this behavior repair with signoff.

## 4. Separate code move and render delivery

- [x] 4.1 Move the agreed dispatch methods unchanged into `crates/app/src/dispatch.rs`.
  Add only module/import and necessary visibility plumbing. Keep tests and terminal
  setup in place; review the moved bodies for no logic changes. No cleanup or repair
  in this commit. Run full `task check`; commit this move-only step with signoff.
- [x] 4.2 Add the standalone `renders/85-input-modes.wav` generator through production
  input dispatch and offline Audio, with a silent file target, a direct hit before
  UI delivery and a deferred gate followed by its original release. Check every segment
  for finite samples, no clipping, silence, single onset and expected block-relative
  timing within one sample. Regenerate `renders/84-input-parity.wav` and verify its
  recorded unchanged SHA-256. Run full `task check`; commit the render-test step with
  signoff. WAV files remain local ignored outputs; name them in the owner report.

## 5. Documentation, acceptance and archive

- [x] 5.1 Update `README.md`, `docs/PROJECT.md`, `docs/plans/README.md` and
  `openspec/config.yaml` to show the actual 85 status and scope. Add exact separate
  keyboard/controller passes and render segments to `docs/LIVE-CHECKS.md`; update
  `docs/USER-STORIES.md` for ordered take-file/target behavior. Retain the unresolved
  callback performance issue and unverified hardware facts. Check that every delta
  scenario has its named test, run strict OpenSpec validation, full `task check` and
  `task build`; commit this documentation-only step with signoff.
- [ ] 5.2 Run the owner-requested agent keyboard screen pass in the isolated
  `target/ux` release launch from AGENTS.md; verify no active sound connection first.
  Record observed screens, separately from sound and hardware checks. Ask the
  owner to check keyboard behavior and listen to all named renders.
  Record the result without claiming a controller pass while none is available.
  Physical-only gaps remain explicitly unverified; do not put automatable scenarios
  into `openspec/untested.md`. Any acceptance record is a separate artifact-only
  signed-off commit after full `task check` and git authorization.
- [ ] 5.3 Only after explicit owner acceptance, archive with the OpenSpec CLI and verify
  the resulting main specs retain the original direct-hit scenario plus all new
  scenarios. Run strict validation, main-spec scenario coverage, full `task check`
  and release build again. Commit only the archive/status artifacts with signoff
  and authorization. Do not sync main specs by hand. Do not push or open a PR.

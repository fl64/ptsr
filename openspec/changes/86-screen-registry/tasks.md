## 1. Review and approval

- [x] 1.1 Review the proposal, design and both deltas with the owner. Obtain a new implementation request. Run strict OpenSpec validation, STE lint and `task check` before the separate artifact commit.

The owner approved implementation. The artifacts are in `2eba501`. Implementation started with task 2.1.
Do not mark a task complete before its checks and signed-off atomic commit.
Use the existing `86-screen-registry` branch. Keep accepted closure of 85 separate. Push and MR publication need separate requests.

## 2. Screen registry without valid-state behavior changes

- [x] 2.1 Add the static screen registry and reverse lookups in `crates/input/src/lib.rs`. Use registry codes in focus packing. Test exact codes, keys, titles, groups, uniqueness, every screen and invalid decoding. Preserve bank and mode layouts. Run `task check` before the input-scope commit.
- [x] 2.2 Use registry section names and rules in keyboard and MIDI profile parsing and resolution. Preserve override priority, explicit `none`, saved groups and all FX inheritance cases. Add `existing_screen_bindings_survive_the_registry_change` and `fx_controls_keep_their_priority_through_the_registry` through production keyboard and raw MIDI paths. Keep existing assertions. Run `task check` before the input-scope commit.
- [x] 2.3 Replace duplicate App screen metadata with registry access in group setup, help, learn availability and keymap diagnostics. Keep the explicit Settings compatibility case and presentation order. Add the three screen-help scenario tests with keyboard and raw MIDI actions. Cover effective keys, edit groups, Settings refusal, F12 entry, modifiers and narrow help. Run `task check` before the input-scope commit.

## 3. Explicit invalid-state correction

- [x] 3.1 Remove the default-Pads substitution in shared MIDI state reads. Preserve atomic ordering and process tracked releases independently of invalid state. Reject new actions without creating numbered work. Log one error per observed invalid period outside the callback. Add all four invalid-state scenario tests through real dispatch with fault injection. Cover unknown codes, invalid banks, reserved bits, release variants, disconnect, queue retries, project epochs, pending acknowledgements and recovery. Run `task check` before a separate MIDI-scope commit.

## 4. Verification and owner handover

- [ ] 4.1 Check that each of the nine delta scenarios has its named test. Run `openspec validate --all --strict --no-interactive`, `cargo test -p ptsr --test scenarios`, `task check` and `task build`. Retain the existing behavior assertions and inspect the full diff. Reuse automated checks for the three 85 renders. Record command output and benchmark exceedances without closing earlier timing problems. Include this evidence in the handover documentation commit.
- [ ] 4.2 Update `README.md`, `docs/PROJECT.md`, `docs/plans/README.md`, `docs/LIVE-CHECKS.md` and `openspec/config.yaml` with actual 86 status. Include the separate keyboard, controller and listening checks from the proposal. Keep unreported owner checks open. Run STE lint on changed text and `task check` before the separate documentation commit.

This change adds no sound algorithm or new render. The existing references are `renders/85-input-modes.wav`, `renders/85-mixed-holds.wav` and `renders/85-late-midi.wav`.
Do not claim listening or physical-controller verification from automated results.

## 5. Acceptance and archive

- [ ] 5.1 Obtain owner acceptance and record its exact scope without inventing individual hardware or listening results. Synchronize and archive 86 through the OpenSpec CLI. Check that every added scenario remains in the main specs with its named test. Run strict OpenSpec validation, STE lint, `task check` and `task build` before the separate closure commit. Keep closure of 85 intact.

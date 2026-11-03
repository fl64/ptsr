## Why

Screen names and input rules occur in several separate tables. A missing entry can send a controller action to the wrong screen.
Change 86 gives these tables one source before later changes add screens.

## What Changes

- Define one registry for Pads, Settings, Files, Chop and FX. Use it for screen codes, section names, group identity and shared input rules.
- Keep existing keys, learned assignments, help content, screen layout and sound for every valid input state.
- Keep FX page controls, binding priority and its lack of a separate edit mode.
- Reject an invalid shared MIDI input state instead of treating it as Pads. Log the error outside the audio callback.
- Keep original held-pad releases and disconnect handling available during that error. Resume new input after a valid state arrives.
- Add registry tests and recorded-input tests. Keep all existing behavior assertions.

This is a refactor for valid states, with one explicit correction for invalid MIDI state. It is not entirely behavior-neutral.
These artifacts propose work. They do not report an implementation or owner acceptance.

## Capabilities

### New Capabilities

None. The registry is an internal structure, not a new owner feature.

### Modified Capabilities

- `input-devices`: preserve screen-specific bindings and FX rules. Reject invalid shared state without losing existing releases.
- `screen-help`: preserve screen-specific help and learning entry points when screen metadata comes from one source.

The deltas add compatibility requirements and the invalid-state correction. Existing requirements and scenarios stay in place.

## Impact

Planned code areas:

- `crates/input/src/lib.rs`: registry, group metadata, focus encoding and keyboard section lookup.
- `crates/input/src/midi.rs`: profile section lookup and screen rules in binding resolution.
- `crates/app/src/main.rs`: metadata consumers, help, learning groups and keymap warnings.
- `crates/app/src/dispatch.rs`: screen-learn availability without changes to action handling or pad routing.
- `crates/app/src/midi.rs`: invalid-state rejection, diagnostics and release-safe dispatch.
- `crates/app/src/input_tests.rs`: recorded keyboard and raw MIDI tests through production routing.

The registry adds Rust methods in `input`. It does not change action names, dependencies, profiles or saved file formats.
The audio callback does not change. No new sound feature needs a new render.

The owner requested one MR for closure of 85 and work on 86, from `86-screen-registry`.
Keep closure, these artifacts and implementation steps in separate signed-off commits.
Push and MR publication need a separate request.

## Out of scope

New screens, modes, keys, visual badges, confirmation actions, button holds, relative encoders and SMC-PAD integration are outside this change.
Do not repair unrelated Settings behavior, rewrite the renderer or change the ordering protocol from 85.
Earlier callback budget problems remain open. Recorded input tests do not establish physical-controller acceptance.

## How to verify

Use these owner procedures after implementation, not during this proposal step.

### Keyboard

1. Load a sample on a pad through F6.
2. Open Pads with F2, Chop with F3, FX with F4, Files with F6 and Settings with F12.
3. Open F1 on each screen. Verify that the help names that screen's existing controls.
4. On Pads, open edit with Enter. Verify page, parameter and value keys. Close edit with Esc.
5. On Chop, open slice edit. Verify that it still has only its bounds page.
6. On FX, use Tab, I/K and J/L. Verify block selection, parameter selection and value changes without another edit mode.
7. On Pads, Chop, Files and FX, open F10. Verify the current learning group. Cancel with F10 without saving.
8. On Settings, press F10. Verify that screen learn does not open. Use the MIDI tab for learning.
9. Hold a gate pad. Change screen and bank. Release its original key. Verify that only that pad stops.

### Controller

Use the X-Session Pro with its existing assignments. Do not reset an owner's map for this procedure.

1. Use left Headphone for Files and right Headphone for FX if the current map has those defaults.
2. On Pads, use left Cue to open edit. Verify existing page buttons and parameter knobs.
3. On FX, verify the same page controls on both blocks without opening edit.
4. Open F12 from Pads, Chop, Files and FX. Verify that the corresponding group opens in the MIDI tab.
5. Test one saved screen assignment and one saved global assignment. Verify that neither needs learning again.
6. With a pad controller, hold a gate note. Change screen and bank. Release the note. Repeat with a disconnect.
7. Verify that only the original pad stops in each case.

Physical-controller results remain unverified until the owner reports them. The invalid-state case uses an automated fault test, not controller hardware.

### Listening

This change needs no new render because sound generation does not change. Listen for missing releases, duplicate hits and changed FX response.
The existing regression references are `renders/85-input-modes.wav`, `renders/85-mixed-holds.wav` and `renders/85-late-midi.wav`.
Separate listening results for those files remain unreported. This proposal does not close them.

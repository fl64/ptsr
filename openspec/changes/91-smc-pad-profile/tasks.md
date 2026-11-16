# Tasks

## 1. Evidence and specification

- [x] 1.1 Read the active preset and both pad banks with checked USB replies. Record the backup checksum, Linux port names, observed MIDI bytes and open hardware checks in the design.
- [x] 1.2 Define the Master and Private input routes, physical pad order, guarded encoder setup, safe button roles, controller view and raw replay scenarios.
- [x] 1.3 Align the SMC-PAD plan and current project guide with the measured preset and automatic guarded encoder setup. Keep unverified hardware facts marked.
- [x] 1.4 Check the OpenSpec delta and lint changed English text.

## 2. Guarded hardware setup

- [ ] 2.1 Check one reversible encoder record on the connected device, with exact read-back and restore. Stop and revise the design if the protocol differs.
- [ ] 2.2 Implement active-preset read, backup, relative-record write, read-back, and restoration on failure. Use a test transport for byte-level cases.
- [ ] 2.3 Keep relative actions disabled until setup succeeds. Show a specific status on failed setup. Test disconnect and a changed preset during setup.

## 3. Profile and input

- [ ] 3.1 Replace the incomplete example with a measured Master profile and an empty Private profile. Keep saved owner bindings. Test first setup and restart.
- [ ] 3.2 Map all physical pad rows and both second-bank rows. Test velocity and both note-off forms through raw MIDI and App dispatch.
- [ ] 3.3 Map both encoder banks and five buttons. Test directions, neutral values, CC press/release, aftertouch and two devices in one stream.
- [ ] 3.4 Add `task midi-replay` and recorded streams through the production profile and dispatch path.

## 4. Screen and acceptance

- [ ] 4.1 Show SMC-PAD pad order, controls, both encoder banks, current roles and incomplete setup status in the controller view. Compare the ANSI catalogue with the base.
- [ ] 4.2 Run `cargo fmt --all --check`, changed-crate Clippy, `task test:changed`, and `task test:fast` at the required points. Review `git diff` and commit each completed step.
- [ ] 4.3 Record the owner's physical results for all 16 pads in both layers, both encoder banks, five buttons, SHIFT, PAD BANK, aftertouch and disconnect. Give exact keyboard and controller checks. Do not claim hardware acceptance from replay alone.

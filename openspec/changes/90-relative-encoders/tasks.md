# Tasks

## 1. Specify relative controls

- [x] Record profile syntax, decoding, speed, fine mode, parameter steps and list targets in OpenSpec deltas.
- [x] Record the SMC-PAD measurement limit and all state/order edge cases in the design.
- [x] Run `task openspec:check` and STE lint on changed English text. Commit the text step.

## 2. Decode and route relative MIDI

- [x] Parse relative controls and reject invalid encodings or bindings with file and control names.
- [x] Resolve relative CCs through seeded and learned profile roles without changing absolute controls.
- [x] Track acceleration and a held global fine button per device using receipt times. Clear both on release, invalid-state release or disconnect as applicable.
- [x] Keep a pending-input barrier for steps that can change a later pad route.
- [x] Test raw bytes for all encodings, neutral values, two devices, fine mode and disconnect.
- [x] Run format, changed-crate Clippy and `task test:changed`. Commit the code step.

## 3. Apply parameter and list steps

- [x] Implement the step table for every MIDI `Param` and dynamic edit slot through existing value rules.
- [x] Implement file, project, settings, page and bank step targets with their screen limits and existing preview behavior.
- [x] Test keyboard/relative parity, scoped values, limits, Undo grouping, ordered bank hits and picker routing through production dispatch.
- [x] Compare the ANSI screen catalogue with `origin/master` if rows or hints change. No drawing changed.
- [x] Run format, changed-crate Clippy and `task test:changed`. Commit the code step.

## 4. Review and publish

- [x] Update affected project plans and state the unmeasured SMC-PAD limit.
- [x] Walk each proposal item against tests and visible behavior.
- [ ] Run `task test:fast`, `task specs:check` and `task names` before push.
- [ ] Give the owner separate keyboard and controller checks and the listening step.
- [ ] Archive after acceptance, run `task specs:sync`, and complete the PR checks.

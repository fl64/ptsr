## 1. Specification

- [x] Record the owner decisions, the measurement and the corner cases.

## 2. Hold

- [x] Show the panel while the status reads pressed and hide it on release or a failed read.
- [x] Route pad hits by the hold on both note ranges. Keep each release on its press target.
- [x] Let a visible question take P7 and P8 from both note ranges without a hold.
- [x] Rename the F12 MIDI choice to `Hold panel`. Keep the config key.
- [x] Test each scenario with raw MIDI and status replies.
- [x] Light a hit panel cell for 120 ms. Test the drawing at both sizes.

## 3. Help and documents

- [x] Update F1, `docs/CONTROLLER-PATHS.md`, `docs/SMC-PAD.md`, `docs/VISION.md` and the plan.
- [x] Run format, Clippy, changed tests, the screen catalogue compare and the fast checks.
- [x] Give the owner the build and the check.

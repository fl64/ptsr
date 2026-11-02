## 1. Shared FX screen

- [x] 1.1 Add F4 screen and keyboard navigation with scenario tests.
- [x] 1.2 Move shared effect editing and MIDI page controls to the FX screen; preserve
  saved projects, undo, old bindings and per-slot takeover. Test the scenarios.
- [x] 1.3 Update help, stories and live checks. Run `task check`.
- [ ] 1.4 Run an isolated live UI check of the old workflows and the FX screen.
  Check small and large terminals. Record observed results separately from audio checks.
  Recorded end-to-end tests pass. Live 100x32 FX was checked, but the test session was
  stopped because a separate owner ptsr process already had an audio node. Full live
  verification remains open.
- [ ] 1.5 Ask the owner to check the screen and listen to the effects.

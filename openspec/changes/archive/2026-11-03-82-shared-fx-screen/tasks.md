## 1. Shared FX screen

- [x] 1.1 Add F4 screen and keyboard navigation with scenario tests.
- [x] 1.2 Move shared effect editing and MIDI page controls to the FX screen; preserve
  saved projects, undo, old bindings and per-slot takeover. Test the scenarios.
- [x] 1.3 Update help, stories and live checks. Run `task check`.
- [x] 1.4 Check the old workflows and the FX screen in small and large terminals.
  Recorded end-to-end tests pass. The agent checked live 100x32 FX, then stopped its
  isolated session because a separate owner ptsr process had an audio node. The agent
  did not complete the full live pass. On 2026-11-03 the owner accepted all checks of
  change 82 and authorized closure. The owner acceptance completes this task.
- [x] 1.5 Ask the owner to check the screen and listen to the effects.
  The owner accepted the screen and sound on 2026-11-03.

## 2. X-Session Pro replacement profile

- [x] 2.1 Record physical roles, safe default buttons and reset compatibility in the specs.
- [x] 2.2 Replace the shipped profile and test raw MIDI navigation, preview, pages, FX,
  faders, harmless releases, existing assignments and confirmed reset.
- [x] 2.3 Update the control table and live checks. Run task check and build the release.
- [x] 2.4 Ask the owner to reset the profile and check the physical controller.
  The owner confirmed that the replacement X-Session profile works, then accepted all
  checks of change 82 and authorized closure on 2026-11-03.

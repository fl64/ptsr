# Tasks

## 1. Specification

- [x] 1.1 Create a separate overview change from the owner's request.
- [x] 1.2 Add pattern, input, layout and help deltas before code.

## 2. Screen and projection

- [x] 2.1 Project the visible time page across all bank sections. Cover hidden hits, seams, collisions and empty pads.
- [x] 2.2 Add the F5 screen, bank and view controls, empty-bank switch and learned actions through production routing.
- [x] 2.3 Draw wide pad names, colored bank outlines, the fixed bank index, playhead and safety feedback at both supported sizes.

## 3. Verification and handoff

- [x] 3.1 Add named tests for every scenario, with keyboard and raw MIDI cases for shared actions.
- [x] 3.2 Add the overview to the offline screen catalogue and inspect its 80x26 and 120x40 files.
- [x] 3.3 Run `task check`, `task openspec:check` and `task build`. Record any pre-existing callback budget limit.
- [x] 3.4 Give the owner keyboard, controller, screen and listening steps. Name the existing render.

## Verification

`task check`, `task openspec:check` and `task build` each passed on 2026-11-12. Two later benchmark runs exceeded the 50 % wall-time limit once each during active recording. The first reached 91.35 %. The second reached 121.25 %. Both kept thread CPU time below the limit and recorded a large estimated scheduler gap. A third benchmark run passed. Earlier callback peaks remain an open project limit. The owner has not checked the screen, controller or sound by hand.

## Owner acceptance and closure

The owner accepted the overview and authorized PR publication and squash merge on 2026-11-12. The individual keyboard, controller and listening checks remain unverified. `openspec archive 98l-pattern-overview --yes` added 14 requirements and modified two in the main specifications. The main-spec scenario audit, `task openspec:check` and the post-archive `task check` passed. Merge only after required CI passes.

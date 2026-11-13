# Tasks

## 1. Specify and compare

- [x] 1.1 Record the display changes, conflicting requirements, default decisions, and owner checks in the proposal, design, and spec deltas.
- [x] 1.2 Run OpenSpec validation and STE lint. Commit the specification artifacts before code.
- [x] 1.3 Save the current 80x26 and 120x40 catalogue names and bytes as the comparison base.

## Baseline evidence

The pre-code source at `12a5dad` passed 22 screen tests. Its local manifest has 820 ANSI files. Their names and bytes match the final catalogue of change 121. The manifest stays outside Git at `/tmp/ptsr-122-base-screens.sha256`.

## 2. Quiet cells and clear words

- [x] 2.1 Make empty pad cells quiet. Keep corner marks and the empty-pad test tone. Add keyboard and raw MIDI state tests and catalogue cases.
- [x] 2.2 Change visible scope, source, pattern, help, and MIDI learn words. Keep action IDs, saved groups, and controls. Add scenario tests.
- [x] 2.3 Keep the fixed ten-cell PLAY slot blank while idle. Test active and idle states at both sizes.
- [x] 2.4 Make the compact Pitch Gate summary conditional. Test all states at both sizes.

## 3. Clip feedback and long labels

- [x] 3.1 Capture the latest clip time on the UI side. Hide the idle label. Test latch, clean hit, repeated clip, and restart with a controlled clock.
- [ ] 3.2 Shorten long names at word boundaries. Test narrow fields, wide graphemes, and safety priority.
- [ ] 3.3 Add the new catalogue entries and compare unchanged areas with the base.

## 4. Check and hand over

- [ ] 4.1 Run focused checks before each code commit. Run `task screens`, `task openspec:check`, `task check`, and `task build` on the final code state.
- [ ] 4.2 Review the diff for sound, input, saved-format, and large-file changes. Give the owner the keyboard, controller, view, and listening checks. Keep unreported hardware and listening results open.

## Implementation evidence

The quiet empty-pad step passed format, Clippy, the file-size gate, 148 TUI tests, two New pad tests, and the empty-wave test. The catalogue test passed and added `pads-quiet-empty` at both sizes. It changed 180 prior frames with empty cells. The existing corner-mark tests passed. The paired New pad test used keyboard events and raw MIDI. The sound path and test-tone view did not change.

The word step passed format, Clippy, and the file-size gate. The source, scope, binding, saved MIDI, 22 prior screen, and new glossary tests passed. A broad app test run found four stale checks. Each passed after correction. The screen catalogue still has 822 files. It changes 712 prior frames because PATTERN help appears on many screens. The controller map uses `ptrn` in six cells, and MIDI learn uses `pattern` with the old action ID.

The idle PLAY step passed format, Clippy, 148 TUI tests, recorded keyboard and MIDI playback tests, and all 23 screen tests. The ten-cell slot keeps its position at both sizes. The screen catalogue still has 822 files and no missing base name.

The CLIP step passed controlled-clock meter tests, 148 TUI tests, Clippy, the file-size gate, and all 23 screen tests. The catalogue has 824 files with no missing base name. It adds `pads-clip-event` at both sizes. The UI-side meter records the latest observed clipped output. The audio callback does no new work.

The Pitch Gate step passed keyboard, raw MIDI, popup, panel, F1, and parameter-page tests. It passed 148 TUI tests, Clippy, the file-size gate, and all 23 screen tests. The catalogue still has 824 files and no missing base name. The detail row stays inside the pad panel at both sizes.

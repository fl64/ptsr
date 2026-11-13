# Tasks

## 1. Specify and compare

- [x] 1.1 Record the display changes, conflicting requirements, default decisions, and owner checks in the proposal, design, and spec deltas.
- [x] 1.2 Run OpenSpec validation and STE lint. Commit the specification artifacts before code.
- [x] 1.3 Save the current 80x26 and 120x40 catalogue names and bytes as the comparison base.

## Baseline evidence

The pre-code source at `12a5dad` passed 22 screen tests. Its local manifest has 820 ANSI files. Their names and bytes match the final catalogue of change 121. The manifest stays outside Git at `/tmp/ptsr-122-base-screens.sha256`.

## 2. Quiet cells and clear words

- [ ] 2.1 Make empty pad cells quiet. Keep corner marks and the empty-pad test tone. Add keyboard and raw MIDI state tests and catalogue cases.
- [ ] 2.2 Change visible scope, source, pattern, help, and MIDI learn words. Keep action IDs, saved groups, and controls. Add scenario tests.
- [ ] 2.3 Keep fixed title slots while blanking idle PLAY. Make the compact Pitch Gate summary conditional. Test all states at both sizes.

## 3. Clip feedback and long labels

- [ ] 3.1 Capture the latest clip time on the UI side. Hide the idle label. Test latch, clean hit, repeated clip, and restart with a controlled clock.
- [ ] 3.2 Shorten long names at word boundaries. Test narrow fields, wide graphemes, and safety priority.
- [ ] 3.3 Add the new catalogue entries and compare unchanged areas with the base.

## 4. Check and hand over

- [ ] 4.1 Run focused checks before each code commit. Run `task screens`, `task openspec:check`, `task check`, and `task build` on the final code state.
- [ ] 4.2 Review the diff for sound, input, saved-format, and large-file changes. Give the owner the keyboard, controller, view, and listening checks. Keep unreported hardware and listening results open.

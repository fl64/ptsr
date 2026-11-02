- [x] 1. Marks are the scope: `Ctrl+A` marks the bank (`pad.mark_bank`), `Ctrl+Shift+A`
  marks all (`pad.mark_all`, `Ctrl+Shift+letter` in the keymap), the old name
  `pad.scope_next` read as `pad.mark_bank`, the scope steps bank / all gone, `Esc` in the
  edit keeps the marks; the panel, the help line and F1; a test for each scenario; commit.
- [ ] 2. The look: a marked pad yellow (fill and frame), the marks in the bank row and in
  the pad line of the hidden grid; a test for each scenario; commit.
- [ ] 3. Reset to clean: `Backspace` in the pad edit, the question, one undo step,
  click-free on a sounding pad (an offline render with the automated checks, a WAV in
  `renders/`); a test for each scenario; commit.
- [ ] 4. README: the Ghostty line `keybind = ctrl+shift+a=unbind`; `docs/USER-STORIES.md`;
  BACKLOG row 20 keeps only copy and move; commit.
- [ ] 5. Live check with the owner.
